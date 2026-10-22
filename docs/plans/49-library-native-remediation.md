---
title: "Plan 49: Library-native remediation and SurrealMX integration"
status: in-progress
date: 2026-10-21
adrs: [ADR-0214, ADR-0215]
review_sources: [docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md, docs/design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md, docs/functional_reviews/plan49-remaining-scope-and-library-integration-2026-10-22.md]
scenario_sources: [docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md]
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [current-work, design, libraries, storage, preparation, numerical-execution, validation]
---

# Plan 49: Library-native remediation and SurrealMX integration

## Purpose and authoring basis

Use the libraries' composed capabilities to remove repeated interpretation, canonical
row-location machinery, mathematical reconstruction and copied orchestration, while
preserving physically typed scientific execution and truthful durable work. The target
is designed as a whole; the review's illustrative remedies do not fix its implementation
shape. SurrealMX **1.0.0 is selected**, including the integration required to use it with
SurrealDB 3.3.4. It is not an optional experiment or contingent on upstream bundling it first.

The [integrated review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md),
[recommendation audit](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md)
and [source coverage](49-source-coverage.md) supply the findings, recommendation corrections,
scenarios, historical evidence and candidate routes. Both new library overviews—
[SurrealDB](../libraries/surrealdb.md) and [Symbolica](../libraries/symbolica.md)—were read
in full before target authoring. Feature follow-ups used their web links, tagged upstream
source and installed pinned APIs, as the operator requested; Context7 is not the source
for these follow-ups.

**Authoring baseline, 2026-10-21. Interface-checked:** source at
`e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05`, with the revised review, audit companion and
supporting audit evidence retained as explicit dirty inputs. The review's original product
baseline is `a791c533eca0f65b195895f071e60cd1e38e121c`, diagnostic publication `0530dcaa1`.
**Proposed:** the integrated target and implementation packages below. Existing probes
retain their original Tested/Measured scope; no product tests or benefit measurements ran
as part of plan authoring. Document validation is recorded separately at the checkpoint.

The standard is Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator 1.5 and
the pse-arrow binding. This is focused foundation assessment integral to authoring, not a
new broad diagnostic audit. Current architecture sections remain authority for implemented
contracts; this plan owns proposed work and its adopted dispositions. Dependency updates
are ordinary technical choices; architecture/interface changes take their existing decision
route. No production implementation is authorized by creating these documents.

### Operator selections and limits

- The operator authorized a hard design pivot using static library assessment as the design
  premise, including newly considered integrations. Probes address concrete risks rather
  than gate every useful feature. Design selection is distinct from implementation acceptance.
- On 2026-10-21 the operator explicitly selected SurrealMX 1.0: “Whatever integration it takes
  for 1.0, we should pursue that.” The version requirement in SurrealDB's manifest is not
  evidence of API incompatibility. Observed adapter differences are implementation work.
- Preserve all 16 durable lanes, independent participants, caller-independent work and no
  new execution daemon. SurrealDB remains canonical; payload/cache formats do not become
  another commit/lifecycle authority. The existing shared managed **database server** remains.
- Preserve physical types, guard creation before algebra, validity/envelope meaning,
  library-owned derivatives, conservation, original-space outcome qualification, native drain
  and value rebinding. Legitimate vendored integration hooks are retained or migrated.
- RC01–RC21 are explicitly accepted target boundaries, recorded below. Acceptance fixes
  these design choices; it does not establish implementation, complete any scientific
  qualification or bypass required ADR/design publication before dependent changes.

## Remaining-scope reassessment — 2026-10-22

The maintainer authorized the combined remaining-plan and library opportunity review and
coordinated **proposed** revisions. The
[2026-10-22 review](../functional_reviews/plan49-remaining-scope-and-library-integration-2026-10-22.md)
is the current design-improvement input alongside the original review/audit. It inspected
clean source `fa7868336`, complete SurrealDB/Salsa/Z3 overviews and actual consumers, with
independent library research and architecture challenge. Its O01–O08 are opportunities,
not renamed DA/F findings. Source findings and historical receipts retain their identities.

**Implemented baseline:** the pause handoff records typed resolved production/lowering,
regional math, result statistics/pruning and locator retirement on substrate v8, shared
runtime requests/supervision, strict physical Python documents, covariance admission,
compiled catalogue and generated maintenance. The earlier authoring-baseline paragraphs
below and in companions describe their original date, not work presumed still unwritten.
Remaining consumer retirement, acceptance and enduring publication remain open.

**Proposed additions and refinements:**

- STO09 composes bounded study scope acquisition and revision-fenced repair pages inside
  SurrealDB; Rust retains scientific policy. Bound total encoded premises, preserve per-item
  progress and notification/cursor fences, and reconcile unknown effects.
- RES07 joins protected block/batch metadata and retains one selection-local DataFusion
  physical expression/pruning predicate across pages. Physical file authentication and full
  decoder admission remain; a metadata join cannot erase their premises.
- PRE01 first updates exact Salsa 0.28.5 and migrates memory inspection to exclusive lane
  accounting with an owned observation. PRE08 makes typed entity fragments and graph facts
  incrementally derived, retaining complete original root obligations and body reuse.
- PRE09 is a bounded configuration/explanation decision, comparing Z3 with finite enumeration;
  TOOL08 adds bounded typed blocker/source/consumer graph explanations using existing graphs.
  Neither installs a new runtime policy engine, graph service or execution daemon.
- QUAL01–QUAL05 consume these changes through representative contract tests. The canceled
  native sweep stays incomplete; no product campaign or benchmark resumed for this review.

The new package rows remain **proposed**. Existing package states and STO01–STO08 evidence
are preserved. Reopening 49a is limited to STO09 operation composition, not a rerun of the
completed engine/transport integration. Regeneration remains the selected data transition.

### Conditional rule decisions from the reassessment

RC01–RC21 remain accepted as recorded. No reversal is required for bounded batching, native
selection-plan reuse, the Salsa patch or pure typed fragments. Two conditional directions
need explicit operator confirmation if their bounded inquiry selects them:

| Rule | Proposed consequence | Decision / route | If not selected |
| --- | --- | --- | --- |
| RC22 | Execute the same Rust study policy in a pinned Surrealism module within a database transaction; change execution/deployment/capability contracts at blueprint §19.3/§20 | pending; I26/GOV02 presents exact module, failure, finite-resource and activation contract before target adoption | STO09 uses batched scope retrieval, pure Rust decisions and guarded application |
| RC23 | Add authored finite configuration exploration and attributed conflict/Unknown results, potentially with a Z3 binding and new public/replay contract | pending; PRE09/I27/GOV02 presents finite vocabulary, alternatives, native identity and evidence before target adoption | Existing concrete requests/routing remain; no configuration feature or SMT dependency is added |

These are conditional proposals, not accepted implementation scope. Their investigations
must not silently fix a new policy authority or public request shape. A third-party patch
update alone needs no ADR. A selected new binding/operation or architecture change follows
its existing ADR/design route before dependent implementation.

### Dependency-ordered next work after explicit execution resumption

1. Reconcile retained-state disposition and representative fixture changes at their existing
   owners; preserve unknown effects. PRE01's Salsa patch/accounting can proceed independently
   of store state and PRE08. Do not repeat the canceled native sweep.
2. Define STO09 complete scope-page/apply contracts and RES07 joined metadata/native-plan
   contracts against actual consumers. Registry/generator changes and client cutover form
   coherent units with targeted controls and immediate retirement of replaced per-item paths.
3. Complete PRE04's residual producer/caller retirement while PRE08 establishes entity-local
   premises, immutable fragments and root import/assembly. MAT keeps consuming immutable
   region products; existing body reuse and original scientific checks remain.
4. Finish RUN/BND/TOOL consumer migration and existing required acceptance. TOOL08 can use
   the established native graph contracts without waiting for PRE08. PRE09/I26 are bounded
   optional decisions and do not block settled work or create a second production path.
5. Reconcile QUAL01 to actual changes, select the remaining representative assembled journeys,
   then scope-end checks/publication. Reuse applicable evidence only at its original scope.

This review/documentation engagement leaves production execution paused. The current handoff
is this section together with the exact earlier [pause evidence](#pause-handoff--requested-closeout-2026-10-21).
No service, dependency, product code or accepted architectural record changed in the review.

**Authoring checkpoint:** the integrated review and coordinated companion revisions are complete
for this engagement. Independent challenge corrections to bounded progress, descriptor coverage,
fragment/root acceptance and historical wording are incorporated. **Tested, documentation only:**
`just turn-end`, documentation publication with `just docs`, and lifecycle validation completed
against the zero-error baseline; these establish document integration, not product behavior.
The earlier unsupported-profile cleanup notices remain at their existing retained-state owner.
Conditional RC22/RC23 decisions and all new implementation/qualification remain open as above.

## Reading route and ownership

| Document | Local design and packages |
| --- | --- |
| [49a: SurrealMX canonical store](49a-surrealmx-canonical-store.md) | STO01–STO09: server/engine/transport integration, typed lifecycle, graph access, schema lifecycle and coordinated cutover |
| [49b: Protected results and selection](49b-protected-results-and-selection.md) | RES01–RES07: exact batch summaries, predicate lowering, protected access and locator retirement |
| [49c: Resolved model and incremental preparation](49c-resolved-model-and-incremental-preparation.md) | PRE01–PRE09: bounded retention, generated handles, resolved identities, indexes and scientific policy products |
| [49d: Symbolica mathematical execution](49d-symbolica-mathematical-execution.md) | MAT01–MAT06: joint demanded arithmetic, derivative composition, library execution and bounded mathematical inquiries |
| [49e: Runtime and native execution](49e-runtime-and-native-execution.md) | RUN01–RUN10: cancellation, shared requests, parameter studies, seam capability, numerical statuses and native lifetime |
| [49f: Python and scientific boundaries](49f-python-and-scientific-boundaries.md) | BND01–BND05: generated vocabulary, physical Arrow custody and covariance provenance/PSD |
| [49g: Contracts, maintenance and tooling](49g-contracts-maintenance-and-tooling.md) | TOOL01–TOOL08: catalogue, conformance/launch consumers, generated operator functions and dependency split |
| [49h: Qualification and publication](49h-qualification-and-design-publication.md) | QUAL01–QUAL05: S01–S11, selected MX/gRPC faults, scientific/boundary integration and scope-end qualification |
| [Source coverage appendix](49-source-coverage.md) | Navigation from recommendations, candidates and all retained artifact families; no duplicate live ledger |

This coordinator alone owns package state, findings, inquiry outcomes and rule decisions.
Companions explain design and acceptance; their package tables have no competing status.
An implementation package can migrate several semantic consumers without taking ownership
of their contracts. Root integration owns shared registry declarations, manifests, codegen,
activation and combined acceptance. Parallel authors/executors coordinate editing surfaces.

Plan 28 retains its E3/E4/E5 and other actual scope; Plan 32 owns document lifecycle conventions;
Plan 40 retains diagnostics qualification. Completed Plan 37/48 supply current context and
decision rationale, not a backlog. This series does not silently close or restart them.

## Combined target and foundation assessment

### Canonical execution: shared server, SurrealMX 1.0 and gRPC

Build the managed server from pinned SurrealDB 3.3.4 source with a narrow MX 1.0 integration,
and move the application SDK/parser family to 3.3.4. The [tagged upstream manifest](https://raw.githubusercontent.com/surrealdb/surrealdb/v3.3.4/Cargo.toml)
declares `surrealmx = "0.27.0"`: a Cargo compatibility range, not an exact `=` pin or an
unbounded minimum. Selecting 1.0 requires a declaration/resolution change. It does not prove
the adapter is incompatible. The inspected [tag comparison](https://github.com/surrealdb/surrealmx/compare/v0.27.0...v1.0.0)
shows read/cursor values changing from `Bytes` to `ByteSlice`; preserve their lifetime and
convert only at the KVS-owned value boundary. Inspected constructor, persistence and commit
interfaces retain their shape. The first implementation step changes the requirement and
builds with only conversions actually needed, preserving the adapter's explicit SI and
existing defaults/error fallback. A second step deliberately adopts SSI, finite controls,
new typed errors and useful engine statistics. gRPC migration is separate from both steps.
The lockfile is updated only for the selected family. See the official
[1.0 upgrade instructions](https://github.com/surrealdb/surrealmx/releases/tag/v1.0.0).

Use memory-resident canonical data with synchronous append-only logging, fsync on every
append and periodic snapshots, under finite deployment-owned capacity. The
[3.3.4 memory adapter](https://raw.githubusercontent.com/surrealdb/surrealdb/v3.3.4/surrealdb/kvs-mem/src/lib.rs)
already wires persistent storage configuration. MX 1.0's memory/transaction-age options
must have explicit refusal and owner semantics. Default ephemeral `memory` is not the
production configuration. Historical `SELECT ... VERSION` is not promised; canonical
domain history, manifests and receipts still are. Server engine capacity includes pinned
versions; server overhead and client streams require their own finite owners.

The final proposed adapter explicitly requests serializable snapshot isolation. Standalone
SSI defaults already exist in both inspected releases; SurrealDB explicitly selects SI,
so a version bump alone does not change isolation. Retain lifecycle guards and qualify
the actual SurrealQL read/index/range paths and 16-lane retry behavior. Existing generated data is disposable under the maintainer's fresh-start decision.
No old MX log is imported or used as rollback input; newly created data retains recovery
and immutable receipt guarantees.

Select gRPC as the single target canonical request transport. Stock gRPC is not merely a
drop-in replacement for the patched WS client: capacity, per-operation context/deadline,
response charging and physical disconnect/drain need lawful integration hooks. Preserve
application/control admission separation, absolute deadlines, dispatch uncertainty and
immutable-receipt reconciliation. The SDK exposes `stream_items()`, but the current payload-drain-owned request path
explicitly refuses it; canonical `bounded_query` still buffers bounded results. Any adoption
must qualify aggregate ownership, late failure and physical drain first;
rows are provisional until statement and transaction success. Frame bounds are not
aggregate response bounds. Transport replacement must not change effect/replay permission.

Registry-generated tagged physical lifecycle/enum types, immutable-field constraints,
transition assertions and typed refusal codes remove parallel interpretation. Preserve
closed-awaiting-manifest and the complete u64 generation range; physical ingestion and
observational AttemptState remain deliberate projections. `FOR UPDATE` does not protect
predicate phantoms, so guard/sentinel rows remain. Pure endpoint associations can use
LIGHTWEIGHT relations; attributed/event-bearing edges use classic ENFORCED relations.
Bounded native inline adjacency replaces extra application caching where its actual edge
shape fits. Necessary commit-order sequencing remains; broad local pacing is investigated.

Select SurrealKit beta.6 for registry-generated schema artifact embedding, frozen rollouts
and schema-source parity tests. It supplies schema lifecycle machinery; generated guarded
functions still own domain maintenance. One fenced fresh initialization and activation replaces the
old store and regenerates data from current sources. There is one production
authority at each point, with no final RocksDB or WS alternate path. The old catalog/payload state is disposable; no compatibility importer or backfill is required.
The existing beta.6 configuration, rollout producer and exact maintenance settlement/
activation owners are foundations to adapt. Their actual clients join the gRPC migration;
the plan does not assume they are absent or already transport-compatible.

### Result execution: canonical blocks, library pruning and protected payloads

Append exact statistics over admitted batches on existing canonical block records, bound
to payload digest, schema and interpretation. The four scalar relation families have
declared keys. DataFusion `PruningPredicateBuilder` skips impossible batches; the canonical
predicate becomes a residual filter before limit, preserving null/NaN/infinity and original
row coordinates. Full protected decoder admission stays. All scalar row-locator writers,
readers, maintenance and tests retire together when the replacement controls pass.

Joined protected metadata pages improve operation shape; incremental SurrealDB streaming
is an unselected integration inquiry, not a property of current canonical reads. Bounded paging
does not move numerical rows into the canonical store. Parquet, projected decoding and membership summaries are bounded
follow-ups with distinct soundness/format questions. Native DataFusion file caches are
bound only at an actually reached source generation or their unused adapters are removed;
protected IPC is not a consumer of those caches.

### Semantic preparation and library mathematics

Use a project-owned resolved, identity-typed physical representation before library algebra.
It retains binder, instance, guard, premise, provenance and unit meaning. Generated relation
handles/membership remove string-policy inventories. Canonical local slots frame reusable
bodies; actual instance binding is separate. Changed body/derived identities have explicit
version/domain frames; rendered text and registration addresses are not identities.

Retain Salsa's granular queries and count LRU, with a finite allowance for admitted live
roots and actual metadata/input/interned/backdating retention. Correct defaults and the
catalogue firewall independently of the representation migration. No nonexistent public
value-holding counter or byte-weighted LRU is assumed. Dependency→consumer indexes include
non-expression rate nodes; current-identity lookups can precede resolved lowering.

Lower admitted arithmetic to Symbolica definitions/aliases and joint demanded output
regions, choosing symbolic local derivative products with correct library chain composition
across intermediate/provider/branch barriers. Deterministic preflight chooses the eligible
route before expensive differentiation; capacity and cancellation refuse rather than change
the artifact opportunistically. C++ compiled execution retains shared code with private
worker scratch. The interpreted evaluator has no public immutable-program/private-scratch
split. `InliningPolicy::Auto` currently equals `Always`; retained subevaluators cannot be
vectorized in 3.0.1. Alias expansion during construction does not supply earlier derivatives.

The broader Symbolica/Numerica ecosystem supplies exact algebra, transformers, coefficient
domains, precision and streaming opportunities. Selected uses and bounded inquiries are
specified in 49d, with no invented full interval certification, universal exact nonlinear
solver, custom arithmetic interpreter or mandatory device backend.

### Runtime, native seam and scientific boundary

One supervisor owns run lifecycle and inherited cancellation/native drain. Shared runtime
requests and preparation products replace host interpretation; host-only operations project
explicitly into the unchanged Version 7 study domain. Parameter-valued Simulation/Fit points
rebind prepared meaning with fresh IC/history rules. Explicit strategy operations retain a
documented lifecycle exception; no new durable strategy or Shooting study feature is added.

Compiler-produced annotation/view decisions and pure start policy feed all relevant consumers.
Settings-dependent native capability owns fixed-point/Picard maps and recovery suitability;
fresh native sessions and independent original residual qualification remain. Uno small-step
code 5 becomes Numerical, its four numerical hunks stay declared, and SQP-LBFGS remains.
FERAL estimates support pre-admission alongside post-factor enforcement; partial allocator
hooks do not erase foreign-envelope coverage. Actual retained rayon workers join before
their charge ends.

Generated enums, identity projection and physical keyed Arrow inputs replace hand-paired
vocabularies and positional side routes. Immutable input custody precedes asynchronous work.
Physical times retain signed/range/sub-microsecond semantics; timedelta is budget-specific.
Qualified covariance is hydrated or fully reconciled with actual fit content/validity.
External origin remains explicit. Independent faer PSD admission accepts singular matrices
and refuses indefinite inputs; a fit id alone certifies neither content nor statistics.

One direction-aware document catalogue feeds schemas, Python types and extension identity.
Conformance/launch/operator tools consume Rust meanings. Product observation/result-selection
contracts stay below tooling; internal module separation is selected before an optional
crate. Enduring architectural publication follows implemented results through confirmed
rule routes.

## Rule decisions

The operator explicitly accepted RC01–RC18, RC20 and RC21 on 2026-10-21 by selecting
“Accept all listed rule changes as written.” RC19 had already been explicitly selected.
GOV01 is complete for these 21 items. The retained-rule alternatives explain the choices;
they are not pending decisions. Acceptance does not select unknown inquiry winners or
authorize production execution. Accepted ADRs remain immutable; GOV02 supplies the
appropriate new/superseding ADR or design revision before implementation changes its contract.

| Rule | Accepted target and consequence | Decision / date | Route before dependent implementation | Retained-rule alternative (not selected) |
| --- | --- | --- | --- | --- |
| RC01 | Permit physically resolved identity-typed specialization, local-slot body framing and versioned derived identities | accepted, 2026-10-21 | GOV02; blueprint §§5.3/7.1, identity ADR route; PRE04/MAT | Keep current no-IR rule; redesign identity remedy before adoption |
| RC02 | Define demanded guarded arithmetic regions and correct symbolic/chain products; deterministic preflight, no whole-group Dualizer | accepted, 2026-10-21 | GOV02; blueprint §§7.1/14; MAT01–MAT05 | Retain wording, document a conforming regional interpretation before adopting it |
| RC03 | Finite live-root/backdating allowance plus actual bounded metadata/value retention and revised defaults | accepted, 2026-10-21 | GOV02; blueprint §§14.3.2/14.4; PRE01 | Current defaults still rotate the reference workload; no claim F09 is fixed |
| RC04 | Exact per-batch statistics and DataFusion residual/pruning replace per-row locators; full protected decoder stays | accepted, 2026-10-21 | GOV02; blueprint §§19.2/20 append contract; RES | Keep locators; predicate lowering alone does not deliver write-volume target |
| RC05 | Registry-generated lifecycle/enum constraints; transition-controlled mutable fences; record-id assertion/index retirement after readers migrate | accepted, 2026-10-21 | GOV02; registry/blueprint §20; STO lifecycle/graph | Keep text/flag physical schema; R2 remains independently useful |
| RC06 | Generated typed refusal codes replace prose conventions; error taxonomy itself retained | accepted, 2026-10-21 | GOV02; generator convention and §23.2 alignment; STO | Prose-dependent callers remain; cannot claim DA-008 resolved |
| RC07 | Settings-dependent capabilities, map representations/recovery and accurate Uno/PETSc matrix | accepted, 2026-10-21 | GOV02; blueprint §§18.1/18.7/18.9; RUN06 | Keep conservative seam and host route; no false extensibility claim |
| RC08 | Declare Uno integration versus four numerical hunks and dense SQP scope, preserving methods | accepted, 2026-10-21 | GOV02; proposed ADR-0157 route; RUN07 | Correct F02 mapping independently; numerical binding declaration remains open |
| RC09 | Keep product detail/selection contracts below tooling; select internal module split, optional crate only after separate evidence/ADR | accepted, 2026-10-21 | GOV02; blueprint §23.1; TOOL05/TOOL06 | Keep placement; do not introduce runtime→tooling cycle |
| RC10 | Describe the actual Simulation/Fit value-binding route, not nonexistent PreparedSimulation::rebind | accepted, 2026-10-21 | GOV02; blueprint §13.5; RUN04 | Correct current-state wording but do not claim new binding support |
| RC11 | Correct statrs hold explanation for egobox-moe feature unification | accepted, 2026-10-21 | TOOL07; ordinary manifest-note correction | Leave stale rationale explicitly unresolved; no family upgrade implied |
| RC12 | Shared request/preparation structs, host superset and typed study projection; Version 7 study domain preserved; admit parameter Fit/Simulation points | accepted, 2026-10-21 | GOV02; blueprint §19.3 and affected binding contracts; RUN03/RUN04 | Retain host duplication and current study restrictions; no silent Shooting variant |
| RC13 | State explicit conic/recycle/block-initialization strategy lifecycle exception; do not add durable strategy execution | accepted, 2026-10-21 | GOV02; blueprint §§17/19.2; RUN02 | Exception remains unstated; no accidental run/replay claim |
| RC14 | Physical keyed Arrow side inputs, identity-keyed tolerances and precision-preserving time; budget timedelta only where suitable | accepted, 2026-10-21 | GOV02; Python-boundary ADR/blueprint §21; BND02 | Bare/positional routes remain and DA-005 is not closed |
| RC15 | Direct KKT FERAL a-priori estimate with retained complete envelope/post-factor enforcement | accepted, 2026-10-21 | GOV02; blueprint §18.8/native binding route; RUN08 | Retain after-factor accounting and full envelope |
| RC16 | One compiler annotation/view product per mode and explicit pure start policy | accepted, 2026-10-21 | GOV02; blueprint §16.2 and view contracts; PRE05/RUN05 | Host/compiler policy copies remain; no single-owner claim |
| RC17 | Generated guarded operator domain mutations, explicitly separate privileged install/restore/activation | accepted, 2026-10-21 | GOV02; blueprint §20.6/ADR-0164 maintenance contract; STO/TOOL04 | Script remains domain writer; no single-owner claim |
| RC18 | Covariance content/validity reconciliation, explicit external origin and tolerance-governed PSD accepting singular inputs | accepted, 2026-10-21 | GOV02; blueprint §19.8/scientific input contract; BND03/BND04 | Do not label id-only or indefinite propagation certified |
| RC19 | Select SurrealMX 1.0 integration with 3.3.4 shared server and synchronous persistence, preserving lanes and caller independence | accepted, 2026-10-21 | GOV02 records selected storage binding/activation; STO01–STO03 and cutover controls | Not pending: operator explicitly selected integration, not optional comparison |
| RC20 | Replace patched WS with one gRPC transport carrying equivalent bounded admission/deadline/effect/response/drain semantics | accepted, 2026-10-21 | GOV02; supersede/reconcile retained WS target from Plan 37 and transport owner; STO | Retain lawful WS transport; MX 1.0 storage selection still stands |
| RC21 | After the minimal MX 1.0 version integration, explicitly select SSI in the adapter and preserve actual conflict/retry, guard and visibility contracts | accepted, 2026-10-21 | GOV02; blueprint §20 transaction contract and selected native storage binding; STO02 | Keep explicit SI with current guards; MX 1.0 upgrade, crash-safe persistence, bounds and statistics still proceed |

## Execution sequence and shared prerequisites

This is the original sequence and contract rationale. The dependency-ordered next work in
the 2026-10-22 reassessment above accounts for implemented prerequisites and proposed additions.

1. GOV01 has recorded the accepted rule decisions; GOV02 schedules concrete decision/design routes.
   RUN01 horizon lineage, RUN07's truthful code mapping and non-contract local investigation
   can proceed independently once implementation is separately authorized. PRE01 defaults/
   bounded-retention work does not depend on PRE04's resolved representation.
2. STO01 first establishes the smallest MX 1.0 upgrade with existing SI. STO02 deliberately
   adds SSI, finite controls, typed failures and statistics; it does not infer them from the
   dependency bump. STO storage integration and gRPC lifecycle design share capacity/effect contracts, not a
   requirement to deploy together before either can be tested. Typed refusals can precede
   lifecycle migration. Shared registry vocabulary supplies BND01; PRE02 handles supply
   RES declared-field access. TOOL01 catalogue has an early membership/direction contract
   and later complete boundary consumer adoption; do not make that distinction a cycle.
3. PRE03 current-identity indexes can precede PRE04. PRE04 supplies final body/derived
   identity and instance-binding contracts consumed by MAT and the resolved-index migration.
   MAT may develop over existing stage programs, retaining current chain propagation until
   the correct replacement is qualified. PRE05 policy products precede RUN05 consumer removal.
4. RUN02's paired finish interface precedes RUN03; RUN03 working request/preparation entry
   precedes RUN04 parameter studies. RUN06's working map seam precedes removing Python's
   KINSOL choice from the host migration. BND02 consumes the runtime's admitted-input custody
   boundary, not a new Python numerical authority.
5. STO schema/freeze/maintenance contracts precede TOOL04 script mutation cutover; RES block
   summaries and migrated payload readers precede locator deletion. Product selection/detail
   contracts precede TOOL05 extraction. All changed producers, consumers, fixtures and old
   paths move together as targeted replacement tests pass—no compatibility production path.
6. Test the actual MX/gRPC migration mechanism before activating the replacement. Coordinate
   freeze, drain, fresh initialization and one activation; regenerate current fixtures. Native
   and boundary migrations retain existing support until their actual replacements pass.
7. Once all selected functional packages and inquiry outcomes are integrated, run QUAL02–05
   assembled journeys and scope-end hygiene/manual checks. Measurements substantiate only
   quantitative claims. Publication does not convert unexamined scientific scope into a pass.

## Package state

Remaining rows distinguish implementation progress from package acceptance. Package
acceptance is explained at its companion owner. GOV01 records completed operator decisions;
GOV02 is the remaining coordinator decision/publication route.

| Package | Status | Owner | Required capability / completion |
| --- | --- | --- | --- |
| GOV01 | done | coordinator | Operator explicitly accepted RC01–RC21, 2026-10-21; no product implementation claimed |
| GOV02 | in-progress | coordinator | ADR-0214/blueprint239 cover storage; proposed ADR-0215/blueprint241 publish remaining accepted target and I24 before dependent changes. Final implemented publication remains open. |
| STO01 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: pinned source-built3.3.4/MX1.0, owned adapter conversions and actual source units; store Outcome. |
| STO02 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: SSI, synchronous persistence/finite controls, authenticated native statistics, reader/fault controls; store Outcome. |
| STO03 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: generated v7 phases/constraints/refusals, migrated consumers, terminal manifest event and exact witnesses; store Outcome. |
| STO04 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: bounded gRPC/physical generations, original clocks/effects, control progress, reconnect and removed custom WS; store Outcome. |
| STO05 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: native records/ranges/graphs, key assertions before index retirement, nodes-first ENFORCED edges and bounded fallback; store Outcome. |
| STO06 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: commit-order paging and sixteen-lane progress. I01 retains necessary source/sequence pacing with explicit rationale; store Outcome. |
| STO07 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: fresh functional/reference profiles, frozen schema activation, interrupted initialization, new-data recovery/backup/restore; store Outcome. |
| STO08 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: selected assembled durability/native/Python/interruption journeys, repaired hygiene, full Clippy/manual/feature checks and installed handoff; store Outcome and limits. |
| STO09 | proposed | [49a](49a-surrealmx-canonical-store.md#sto09--bounded-study-operations) | O01: complete bounded scope retrieval and revision-fenced repair with per-item recovery; existing engine/transport evidence retained. |
| RES01 | in-progress | [49b](49b-protected-results-and-selection.md) | Local package contract and acceptance at the linked owner |
| RES02 | in-progress | [49b](49b-protected-results-and-selection.md) | Local package contract and acceptance at the linked owner |
| RES03 | in-progress | [49b](49b-protected-results-and-selection.md) | Scalar locators and consumers retired on v8; selected canonical controls passed; assembled result acceptance remains with RES06/QUAL. |
| RES04 | superseded | [49b](49b-protected-results-and-selection.md) | Maintainer waived existing-data preservation; no historical backfill is required |
| RES05 | in-progress | [49b](49b-protected-results-and-selection.md) | Local package contract and acceptance at the linked owner |
| RES06 | proposed | [49b](49b-protected-results-and-selection.md) | Local package contract and acceptance at the linked owner |
| RES07 | proposed | [49b](49b-protected-results-and-selection.md#res07--joined-metadata-and-retained-native-selection) | O02/O03: joined protected metadata and selection-local physical predicate/pruning reuse, preserving exact descriptor coverage and custody. |
| PRE01 | in-progress | [49c](49c-resolved-model-and-incremental-preparation.md) | Existing retention/firewalls have bounded evidence; O04 adds proposed Salsa 0.28.5/exclusive accounting before new incremental work. |
| PRE02 | in-progress | [49c](49c-resolved-model-and-incremental-preparation.md) | Local package contract and acceptance at the linked owner |
| PRE03 | in-progress | [49c](49c-resolved-model-and-incremental-preparation.md) | Local package contract and acceptance at the linked owner |
| PRE04 | in-progress | [49c](49c-resolved-model-and-incremental-preparation.md) | Local package contract and acceptance at the linked owner |
| PRE05 | in-progress | [49c](49c-resolved-model-and-incremental-preparation.md) | Local package contract and acceptance at the linked owner |
| PRE06 | proposed | [49c](49c-resolved-model-and-incremental-preparation.md) | Local package contract and acceptance at the linked owner |
| PRE07 | proposed | [49c](49c-resolved-model-and-incremental-preparation.md) | Local package contract and acceptance at the linked owner |
| PRE08 | proposed | [49c](49c-resolved-model-and-incremental-preparation.md#pre08--typed-incremental-fragments) | O05/I07: entity-local tracked premises and immutable fragments with complete root assembly; no claim of affected-only global validation. |
| PRE09 | proposed | [49c](49c-resolved-model-and-incremental-preparation.md#pre09--finite-configuration-exploration-decision) | O06/I27: bounded operation and Z3-versus-enumeration decision; RC23 pending before new functional target/binding adoption. |
| MAT01 | in-progress | [49d](49d-symbolica-mathematical-execution.md) | Local package contract and acceptance at the linked owner |
| MAT02 | in-progress | [49d](49d-symbolica-mathematical-execution.md) | Local package contract and acceptance at the linked owner |
| MAT03 | in-progress | [49d](49d-symbolica-mathematical-execution.md) | Local package contract and acceptance at the linked owner |
| MAT04 | in-progress | [49d](49d-symbolica-mathematical-execution.md) | Local package contract and acceptance at the linked owner |
| MAT05 | proposed | [49d](49d-symbolica-mathematical-execution.md) | Local package contract and acceptance at the linked owner |
| MAT06 | proposed | [49d](49d-symbolica-mathematical-execution.md) | Local package contract and acceptance at the linked owner |
| RUN01 | in-progress | [49e](49e-runtime-and-native-execution.md) | Local package contract and acceptance at the linked owner |
| RUN02 | in-progress | [49e](49e-runtime-and-native-execution.md) | Local package contract and acceptance at the linked owner |
| RUN03 | in-progress | [49e](49e-runtime-and-native-execution.md) | Local package contract and acceptance at the linked owner |
| RUN04 | in-progress | [49e](49e-runtime-and-native-execution.md) | Local package contract and acceptance at the linked owner |
| RUN05 | in-progress | [49e](49e-runtime-and-native-execution.md) | Local package contract and acceptance at the linked owner |
| RUN06 | in-progress | [49e](49e-runtime-and-native-execution.md) | Local package contract and acceptance at the linked owner |
| RUN07 | in-progress | [49e](49e-runtime-and-native-execution.md) | Local package contract and acceptance at the linked owner |
| RUN08 | in-progress | [49e](49e-runtime-and-native-execution.md) | Local package contract and acceptance at the linked owner |
| RUN09 | in-progress | [49e](49e-runtime-and-native-execution.md) | Independent library-default criteria, profile/replay and retained results implemented; focused native controls passed; wider Python/scientific qualification remains open |
| RUN10 | in-progress | [49e](49e-runtime-and-native-execution.md) | Local package contract and acceptance at the linked owner |
| BND01 | in-progress | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| BND02 | in-progress | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| BND03 | in-progress | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| BND04 | in-progress | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| BND05 | proposed | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| TOOL01 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL02 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL03 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL04 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL05 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL06 | done — retain hub and internal tooling module | [49g](49g-contracts-maintenance-and-tooling.md) | Two actual steady/fit and bounded-lifetime inquiry controls passed; no replacement substrate or separate crate selected. |
| TOOL07 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL08 | proposed | [49g](49g-contracts-maintenance-and-tooling.md#tool08--bounded-graph-explanations) | O07: typed immediate-blocker and bounded source/consumer explanations over current canonical graph identities, with explicit partiality. |
| QUAL01 | in-progress | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |
| QUAL02 | proposed | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |
| QUAL03 | proposed | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |
| QUAL04 | proposed | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |
| QUAL05 | proposed | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |

## Finding dispositions

The following records current correction dispositions. Review “Addressed” and audit
“Retain” are not completion. Partial/challenged diagnoses retain their limits; the store
Outcome supplies the selected STO evidence without closing combined downstream obligations.

| Finding | Review scenarios | Disposition | Work owner | Required evidence or question |
| --- | --- | --- | --- | --- |
| DA-001 | S11 | open — proposed route | PRE04 | Resolved physical identity before library algebra |
| DA-002 | S11 | open — proposed route | PRE04 | Versioned derivation frames and declared relationships |
| DA-003 | S11 | open — proposed route | PRE02 / PRE04 | Generated grammar/role metadata; no formatting-derived meaning |
| DA-004 | S04 | open — store portion implemented/tested | STO / BND01 | Store native phases/projections pass; complete shared boundary vocabulary remains BND01. See store Outcome. |
| DA-005 | S08 | open — proposed route | BND02 / I20 | Physical keyed side inputs; unit resolver is a separate project decision |
| DA-006 | S07 | open — proposed route | PRE05 / RUN05 | Compiler annotation product, deliberate per-mode differences |
| DA-007 | S04 | open — store portion implemented/tested | STO lifecycle / TOOL04 | Native transition/holder/release controls pass with race guards; broader generated operator migration remains TOOL04. |
| DA-008 | S05 | resolved — Implemented and Tested | STO typed refusals | Registry-native codes, typed exhaustion and distinct dispatch/cancel/replay/effect controls; unknown/forged errors fail closed. See store Outcome. |
| DA-009 | S10 | open — proposed route | RUN02 / RUN03 | Discriminant-paired requests and outcomes |
| DA-010 | S08 | open — proposed route | BND03 / BND04 | Hydrate/reconcile complete covariance; independent PSD admission |
| DA-011 | S04 | open — proposed route | TOOL01 | Direction-aware actual compiled catalogue and import identity |
| DA-012 | S03 | open — proposed route | RUN01 / RUN02 / RUN03 | Shared supervisor; explicitly state strategy exception, no automatic new durable feature |
| DA-013 | S09 | open — proposed route | PRE05 / RUN05 | One pure predicted-start/fallback owner |
| DA-014 | S09 | open — proposed route | RUN04 / I22 | Parameter-valued Fit/Simulation selected; other overlay/history questions bounded |
| DA-015 | S10 | open — proposed route | RUN03 | Runtime admission, typed host projection and deployment owner |
| DA-016 | S07 | open — proposed route | RUN06 | Settings-dependent map/recovery seam with original residual |
| DA-017 | S07 | open — proposed route | RUN06 / TOOL07 | Advertised capability equals admitted execution and architecture |
| DA-018 | S07 | open — proposed route | RUN07 / I23 | Challenge integration-as-fork diagnosis; declare four numerical hunks; no forced method loss |
| DA-019 | S04 | open — store portion implemented/tested | STO lifecycle / TOOL04 | Fresh installation/activation and exact event witnesses pass; broader generated guarded operator mutations remain TOOL04. Historical data preservation superseded. |
| DA-020 | S10 | open — proposed route | TOOL05 / I21 | Product contracts before tooling; optional tracing/crate outcome |
| DA-021 | S10 | open — proposed route | RUN03 | Test eligibility from explicit inputs without full runtime |
| DA-022 | S01 | open — proposed route | PRE01 / RUN08 / I06 / I16 | Actual retention and allocator domains; project structure sizing remains bounded lead |
| DA-023 | S04 | open — proposed route | TOOL07 / QUAL05 | Documentation follows selected implemented changes through decision route |
| DA-024 | S10 | open — proposed route | TOOL07 / I24 | Purpose decision on native child and pse.canon.v2; no automatic deletion |
| DA-025 | S05 / S06 | open — store portion implemented/tested | STO / RES / TOOL04 | Necessary participants retained; pacing inquiry and key-index correction completed. Result writes/reclamation and broader tooling remain RES/TOOL04. |
| DA-026 | S06 | open — proposed route | RES01–RES06 | Batch summaries + residual semantics replace scalar locator writes |
| DA-027 | S11 | open — proposed route | PRE04 / MAT01–MAT05 | Versioned resolved body frames and no text re-expansion |
| DA-028 | S11 | open — proposed route | MAT01–MAT05 / I11 | Guarded regional realization; prior fixture did not establish universal magnitude |
| DA-029 | S01 | open — proposed route | PRE03 / PRE04 | Dependency→consumer graph including rates and complete memberships |
| DA-030 | S01 / S02 | open — proposed route | PRE01 / PRE06 | Retention/default cause corrected; audit pointer/key lifetime separately |
| DA-031 | S07 | open — proposed route | PRE05 / RUN05 | One compiler view choice consumed by relevant modes |
| DA-032 | S06 / S11 | open — proposed route | PRE02 / RES | Generated declared handles and membership |
| F01 | S03 | open — proposed route | RUN01 / RUN02 | Caller cancellation, deadline and native join |
| F02 | S07 | open — proposed route | RUN07 | Uno code 5 Numerical, all named constants |
| F03 | S05 | resolved — Implemented and Tested | STO refusals / I01 | Typed retry exhaustion/fresh decision under original clock; commit-order cursor and sixteen-lane controls pass. Necessary sequence/source guards retained. See store Outcome. |
| F04 | S03 | open — proposed route | RUN10 / I14 | Retained actual worker join; global reach investigated |
| F05 | S09 | open — proposed route | RUN09 / I18 | Cosine gtol distinct from absolute scaled stationarity |
| F06 | S06 | open — proposed route | RES cache package / I02 | Reached source-generation cache or remove only unused adapters |
| F07 | S04 | resolved — Implemented and Tested | STO graph/access | Consumers use native records/ranges/graphs; direct CRUD key/id/u64 and access-plan controls precede redundant key-index removal. Engine scans remain distinct from page bounds. |
| F08 | S04 | open — proposed route | TOOL07 | Correct stale statrs note without pretending type-crossing defect |
| F09 | S01 / S02 | open — proposed route | PRE01 | Production-default live-set retention, reuse/churn/oversize |
| F10 | S08 | open — proposed route | BND03 / BND04 | Content-sensitive fit provenance; indefinite refused and singular PSD admitted |

## Inquiry outcomes

Each question has a route and a revealing outcome. Pending adoption is not a second
implementation path; either the selected mechanism replaces its predecessor or it is not
adopted. A no-adoption conclusion can complete an inquiry without resolving an unrelated
finding. Technical library capability is judged on merit; licenses are not rejection reasons.

| Inquiry | State | Owner | Question and evidence | Adoption / no-adoption consequence |
| --- | --- | --- | --- | --- |
| I01 | resolved — retain necessary pacing | STO / QUAL03 | Actual sixteen-lane mixed work attributes queue/hold/conflicts and proves control progress; source creator/reclaimer and sequence commit dependencies remain. See store Outcome. | Keep ProblemTurns and shared source-creator/sequence guards. No safe excess-scope replacement established; no throughput improvement claimed. |
| I02 | resolved — remove unused adapter | RES / F06 | The reached protected result reader owns an instance-keyed IPC reader; no production DataFusion file source reaches the removed native file-cache adapter. The Parquet authoring importer uses a separate direct reader. | Retain active analysis/preparation caches. Revisit when a named DataFusion file-source consumer supplies immutable version/etag or owned generation and eviction; qualify replacement, cold/hit and cancellation there. |
| I03 | resolved — retain min/max | RES | Existing facts own min/max, null and row counts, not exact bounded multi-value membership sets. Native contained means all/none of the queried literals; unknown must remain unknown. | No parallel membership index. Revisit an already-owned exact bounded dictionary/set with proved complete coverage, null/NaN semantics and same-file binding. |
| I04 | resolved — retain full decoder | RES | A projected IPC reader omits fields and changes its schema; full footer schema alone does not admit omitted batch buffers or their semantics. | Keep full protected validation and batch admission, then engine projection/residuals. Revisit a measured wide payload with immutable prior full-batch admission, predicate/key closure and omitted-column corruption controls. |
| I05 | resolved — retain protected IPC | RES | Native Parquet page/row selection exists, but current protected result descriptors, checksums, footer validation and decoder consume IPC. Parquet authoring import is a separate boundary. | Revisit a reached workload with measured page/range benefit under full-file integrity, explicit format/interpretation decision, IEEE bit preservation, full admission, conservative residuals and finite decoded memory. No comparative speed claim. |
| I06 | resolved — retain shared census | PRE01 | Salsa0.28.4 invokes heap_size independently per input; it cannot deduplicate allocations shared by retained inputs and generations. | Keep the actual shared-Arc census; local callbacks do not replace its ownership set. |
| I07 | open — revised target proposed | PRE08 / PRE06 | O05: typed resolved production now permits entity-local tracked fragments and selective graph facts; current root specialization/projection still recomputes selected-root derivation. | Develop typed fragment/import contract and compare incremental with clean assembly; preserve original contributions/closure and unchanged body reuse. No benchmark failure prerequisite or speed claim. |
| I08 | resolved — retain independent publication lanes | PRE06 | Salsa0.28.4 Storage::cancel_others cancels and waits for storage clones during mutation. Existing historical/current workspaces retain independent publication ownership. | No new shared read-handle seam is selected; reconsider a concrete immutable-revision benefit with fresh ownership controls. |
| I09 | resolved — retain reached pure Flights | PRE06 | Existing exact-key flights retain producer clocks, independent waiter stop and actual exit; quick_cache0.6.24 directly awaits a supplied loader. | Retain the reached flight contract rather than add an adapter to recreate it. Native drain is never cache-flight completion. |
| I10 | resolved — no additional compiler persistence | PRE06 | Existing qualified artifact recipes restore reconstructible products with version/receipt readmission; no distinct reached cold-work need is established. | Keep those owners and in-process correction; add no persistent compiler cache or lifecycle authority. |
| I11 | resolved — retain scalar formals and aliases | MAT06 | Which function-shaped formal/retained definition path preserves cross-process identities, derivative expansion and compiled export? | Adopt the exact supported post-admission path; otherwise canonical scalar slots plus selected alias expansion remain. |
| I12 | resolved — retain scalar profiles | MAT06 | Does JIT/SIMD/CUDA improve an eligible repeated/batched workload with declared float, cancellation and scratch semantics? | Select and qualify a specific realization; otherwise retain current scalar interpreted/C++ eligibility, no custom interpreter. |
| I13 | resolved — retain scoped library algebra | MAT06 | Do exact solving, polynomial/coefficient domains, transformers or term streaming remove a concrete preparation burden? | Adopt scoped library work preserving coverage/domain/remainder and cleanup; otherwise keep current lowering and record conditions for reconsideration. |
| I14 | resolved — scoped teams and retained upstream setup | RUN10 | Which production library actually reaches rayon global pool and who owns its embedding-host initialization? | Control reached work under admitted shared owner or report incompatibility; do not eagerly create/pin a global pool without a consumer. |
| I15 | resolved — retain envelope | RUN08 | Can PETSc/KLU hooks retain original allocation owner/size across free/realloc and process initialization, with bypasses identified? | Use domain-specific accounting with remaining envelope; otherwise retain the complete foreign envelope. |
| I16 | resolved — retain admitted ownership and sizing | RES / RUN08 | Arrow claim tracks existing backing with an infallible reservation and replaces a shared per-buffer owner; it cannot replace fallible preflight or imported lease custody. Project HeapUsage and generated rows already account their actual structures; shared result facts retain one owner. | No additional pool or sizing crate. Revisit a concrete unsupported allocation/graph lifecycle with fallible admission, no double charge, last-alias custody and measured overhead; DataFusion structure sizing alone is not an admission proof. |
| I17 | resolved — fresh sessions | RUN06 | Does KINSOL neutral reset match independently fresh sessions including history and errors? | Adopt only qualified reset or record warm history in semantics; otherwise fresh sessions stay. |
| I18 | resolved — fixed independent criteria | RUN09 | Use the pinned non-minpack library defaults explicitly: separate relative cost, relative step and cosine criteria, each 30 times f64 epsilon, independently of physical stationarity and covariance qualification. Five actual LM controls passed, including the original-policy comparison and cosine/stationarity disagreement; retained public result reopening also passed. | Keep exact profile/replay identity and effective work caps. Wider scientific and Python qualification remains RUN09/QUAL scope; no public tuning knobs or speed claim. |
| I19 | resolved — identical selected catalogues | TOOL01 | Does the actual direction-qualified exported catalogue vary across supported feature builds? | Actual default and linked-native force-validated extensions share the generated catalogue identity; future exported-shape variation must generate its actual identity. No all-feature-powerset claim. |
| I20 | resolved — reuse existing physical owner | BND02 / PRE02 | Actual admitted Ports own coordinate identity/quantity/unit; BindingQuantity and CanonicalConversionPlan already resolve complete registered physical meaning. Tolerances/characteristic scales require Delta conversion, not point offsets. | Use those owners with the new keyed relation contracts and immutable snapshots; no independent unit table/resolver. BND02 integration and qualification remain open. |
| I21 | resolved — retain hub and internal module | TOOL06 | Actual no-layer/current-hub/candidate-layer steady and fit results/covariance agree bitwise. Attributed spans reach the candidate, but the actual native.worker phase has no corresponding candidate span; bounded state drains on completion, future drop and panic. A separate tooling crate would currently require a runtime/catalogue dependency seam; no independent build benefit is established. | No Layer substrate or new crate adoption. Keep the current hub and corrected internal dependency boundary; no speed or overhead improvement is claimed. |
| I22 | resolved — no additional adoption | RUN04 / MAT06 | Reached Simulation/Fit overlays are parameter-valued; typed point replacement does not define IC constraints, schedule topology or observation provenance. Existing IDAReInit/IDASensReInit and bounded neutral storage reuse are already consumed. Current structural scopes already serve their reached consumers. | Retain those paths. Revisit a named new input meaning or measured compatible setup burden, with separate identity/history decisions and fresh-contract reset controls. No universal bit equivalence or performance benefit is established. |
| I23 | resolved — retain declared binding | RUN07 | Is there a technically equivalent upstream Uno path preserving SQP-LBFGS and the four numerical-hunk meanings? | Upstream when equivalent; otherwise declare retained numeric integration. Different IPM/PETSc methods are not substitute remediation by name. |
| I24 | decided — implementation open | TOOL07 | Does native child lane or pse.canon.v2 have an explicitly named bounded exploration purpose? | Operator selected retain native-child for bounded provider-free process-isolation experiments and retire unused relation canonicalizer/API/tests; preserve independent shared allocation and identity contracts. Execution remains TOOL07. |
| I25 | resolved — no hot-start adoption | MAT06 | Can hot_start reuse an identity-bearing previous scheme on a non-direct path with a concrete preparation benefit? | Adopt only with scheme in identity and correct derivative semantics; direct-translation alias target remains and cancelled truncated optimization is never retained. |
| I26 | open | STO09 / GOV02 | O01: would same-source Rust policy in Surrealism remove enough coordination to justify pinned WASI/module/deployment/resource semantics? | Batched Rust-policy operations are selected proposed baseline; RC22 remains pending for a later module pivot. No independent SQL policy copy. |
| I27 | open | PRE09 / GOV02 | O06: compare a concrete finite configuration/explanation operation using Z3 with bounded enumeration over owned capability facts. | Adopt only after explicit RC23 functional/binding selection; otherwise retain concrete request routing and record a reasoned no-adoption. |
| I28 | resolved — retain bounded canonical pages | STO09 / RES07 | Current canonical bounded_query materializes responses; local stream_items refuses payload-drain-owned requests. | Joined bounded pages are proposed. Reopen streaming for a named consumer only with original deadlines, aggregate bytes, late failure and physical drop/drain. |
| I29 | resolved — no notification authority added | STO09 / TOOL08 | LIVE/changefeeds/async events provide observations with retention/delivery/import limits; current durable cursors/recovery remain necessary. | Reopen an actual wakeup/operator consumer with bounded catch-up; notifications never replace exact receipts, canonical claim or native drain. |

## Verification, checkpoint and completion

**Proposed at authoring:** companions' targeted tests and QUAL assembled qualification.
Product implementation and qualification were not_run during authoring; the store Outcome
below records later executed scope. The review's original memory-engine and guard-free measurements are historical
only. The selective msgspec identity-hook test settles its narrow codec question, not the
complete boundary. G4/G6 and scientific/profile coverage retain their actual limits.

Authoring completion requires complete source routing, consistent shared contracts,
bounded unresolved inquiries, coherent sequencing, recorded rule decisions, lifecycle
bindings/navigation and a successful documentation build or an exact external blocker.
Implementation completion additionally requires every selected package, all producer/
consumer migration and deletion, required assembled qualification, confirmed decision
publication and evidence-backed finding dispositions. No plan closes another plan by analogy.

**Authoring checkpoint, 2026-10-21 (historical):** ten proposed documents were authored, including the coordinator,
eight companions and source coverage. MX 1.0 selection is explicit; its minimal dependency
integration is separate from deliberate SSI/control/statistics adoption and gRPC migration.
Focused independent design advice found no outstanding material gap after the covariance
tolerance and existing SurrealKit ownership clarifications. All 21 rule decisions are accepted
and GOV01 is complete; GOV02 and implementation/qualification packages remain proposed.
Product source and the user's library documents were unchanged during authoring.

**Tested, documentation only:** publication builds the series with scoped search; lifecycle
validation and source routing cover 56 packages, 42 findings and 25 inquiries as 123 native
items at one owner: GOV01 complete, 122 items open. Relative link targets are present. Initial metadata vocabulary/table
formatting mistakes were corrected. The normal turn-end/ready entry was blocked before
checks by the existing shared-registry fenced-recovery requirement; adr-index/formatting
passed with the previously documented isolated test registry. Shared-state recovery remains
unchanged. These checks establish documentation integration, not product, migration,
native, numerical or performance qualification. All product acceptance remains Proposed.

### Execution checkpoint — fresh Plan49a target, 2026-10-21

The maintainer authorized all STO01–STO08 and selected fresh regeneration of existing
data. All eight store packages are done; the store Outcome below owns actual acceptance.
Selected functional journeys, final feature/manual checks and installed handoff pass.
The coordinator remains in progress for its other companions. RES04 is superseded by the
fresh-data decision; no historical preservation/backfill is required. RES01–RES03, full
PRE02 and broader TOOL04 are not prerequisites by association.

The final MX1.0/gRPC/v7 schema is installed in fresh functional and reference profiles with
their original numeric allocations. Schema publication, native metrics/faults, new-data
recovery, persistent maintenance, sixteen actual native owners, independent caller/worker
continuation, Python consumers and entered pull/analysis SIGINT are qualified at their
selected boundaries. Typed unknown effects, fenced absence proofs, physical response drain
and protected history remain explicit. Necessary ProblemTurns/source/sequence guards and
attributed ENFORCED relations are deliberate retained choices, not deferred failed remedies.

ADR-0214 and blueprint239 preceded dependent implementation; blueprint240 publishes selected
acceptance. Formal ADR status remains proposed. The shared fixture registry's preserving
recovery retained its existing records. Failed/unknown fixture outcomes retain pins and
incomplete SQL dispositions even after authenticated physical drain; they are not rewritten
as successful. The functional profile is authenticated/open; reference and private
qualification profiles are stopped/drained. The consumed companion remains retained for
the active coordinator and downstream boundary/maintenance work.

## Outcome — Plan49a canonical store

**Implemented and Tested, 2026-10-21:** STO01–STO08 are complete, including selected
assembled qualification, scope-end static/manual acceptance and installed handoff.
The Plan49 series remains in progress for its other companions.
ADR-0214 remains proposed under the separate decision-PR rule. Other companions retain
their packages and findings; selected store acceptance does not accept their implementation.

One source-built SurrealDB3.3.4 server uses exact SurrealMX1.0.0, explicit SSI, synchronous
AOL/EveryAppend and periodic snapshots. Finite engine memory, transaction age, queues and
outer process allocations remain distinct. Authenticated existing metrics expose engine
statistics and reader pins. The pinned server source, patch inputs, resolved lock and binary
are retained in its installation receipt. The final installed server is
`v3.3.4-mx1.0.0-baedb00f7f55498e9879`, binary SHA-256
`4da6ffce33f240eb5dee5fe24e46de7aa225b7cddfe074d7f0e09fb0a273a500`.

The sole production request path is bounded gRPC over an owned physical HTTP/2 connection.
Original operation clocks include admission, setup, reconnect, decoding and cleanup;
reserved control admission survives application saturation. Server sessions belong to the
physical connection. A replacement generation restores acknowledged setup and never replays
application mutations. Native typed engine errors and registry-generated domain refusals
retain separate meanings; unknown outcomes remain unknown until receipt settlement.

Registry-generated v7 storage uses five physical phases, native literal unions, immutable
identity, record-key assertions, full-u64 decimal bounds and irreversible compaction.
One synchronous event freezes terminal seed manifests; exact catalog witnesses include it.
Redundant key indexes and obsolete flag/prose/WS consumers are removed. Nodes precede
ENFORCED edges in bounded study ingestion. Native record ranges, graph readiness and
bounded inline dependencies replace applicable access machinery. The fresh publication is
`database/canonical-schema/rollouts/20261021131104__canonical_28f894ce3c844206`;
the replaced publication tree is retired. Existing-data import/backfill and old-AOL rollback
are deliberately absent. Fresh identities fence retired callers; new data retains recovery,
protected-file access and closed backup/restore.

### Verification and evidence limits

**Tested:** the following are local Linux checks against a **zero failure baseline**, using
the pinned nightly, exact source-built server and explicit Arrow force-validation wherever
Arrow is consumed. Native runs use linked solver capabilities and finite supervised
allocations; the reference selection actually enters sixteen native owners. Counts below
are distinct selections, not a sum of overlapping controls. Initial failures and affected
reruns are retained; composite acceptance is not a fresh monolithic suite pass.

| Command / mode and selected scope | Actual result and retained evidence |
| --- | --- |
| Source server locked release build, `storage-mem,cli,mcp`, explicit unwind | Passed; final release 17m07s, zero compiler warnings/errors. Source unary structured-error units: 2 passed. |
| `cargo test --manifest-path <selected-server-source>/Cargo.toml -p surrealdb-kvs-mem --lib --locked`, compile allocation, two jobs, pinned nightly/default debug/unwind | 10/10 passed (seven adapter and three configuration controls), zero compiler warnings; `build/qualification/plan49a-adapter-final/adapter.log`. Owned values across release, getm order/missing entries, conditionals/savepoint/reverse cursor, SSI absent/range conflicts, finite age/reader release, memory refusal and durability/configuration guards. Earlier admission refusal was not_run; no test process started. |
| `just unit-package pse-operations 'test(canonical) and not test(canonical_transport::)' --features pse-operations/canonical-tests,pse-operations/canonical-maintenance`, four nextest threads | 283 run: 278 passed, 5 failed; the exact affected five passed after repairs. Runs `c5cd0719-322f-41c9-9d5a-a05898027146` and `e5e942be-c497-437b-aef3-eca869c551af`. Covers native lifecycle/refusals, holders, compound guards, event tamper, index retirement, SSI point/absent/range conflicts, publication order and sixteen-lane mixed pacing. |
| `just unit-package pse-operations 'test(canonical_transport::)' --features pse-operations/canonical-tests,pse-operations/canonical-maintenance`, controlled peers | 29/29 passed, run `039d4b89-cafe-4725-b6df-99c8d03856e7`. Codec/allocation, provisional rows, late errors, clocks, admission, reconnect and physical drain. Engine API isolated units: 18 passed. |
| Actual retained-client server SIGKILL/restart control | Passed, run `1cc30814-5986-4c66-80a1-254c304cf8e0`; `build/plan49a-restart-55e550e673ae42f6a90da7963449fdb6/complete.json`. Same client restores acknowledged setup, reconciles original receipts and advances publication sequence without mutation replay. |
| `.venv/bin/python scripts/tests/surreal_mx_check.py --binary <selected-server> --strace /usr/bin/strace`, bounded disposable native server | 14/14 passed; final lint-refactored probe rerun also 14/14 passed at `build/qualification/plan49a-mx-hygiene-final-h31hrb9y/probe.log`. Authenticated metrics/public filter, held-snapshot pins, cancel/socket reader drain, connection/session admission, snapshots, synchronous reopen, atomic refusal, acknowledged SIGKILL+torn tail, fsync EIO/unknown/poison settlement, interrupted snapshot replacement, finite memory exhaustion and corrupt framed-log refusal. |
| Actual SurrealKit beta.6 patched gRPC sync/replay parity and frozen forward qualification | Parity 15/15 passed plus forward/cold-catch-up/tamper/missing/fork/gap/owner-death controls; `build/plan49a-tooling-{parity,forward}-20261021-codes/qualification.json`. Published tools/producers remain immutable. |
| `python -m scripts.tests.surrealkit_persistent_fixture_check --directory build/plan49a-persistent-drained --binary <selected-tool> --canonical-binary target/debug/xtask`, exclusive linked mode | 24 passed, cleanup empty. Exact pending nonce/catalog/event settlement, lost activation acknowledgement, forward replay, fresh authority and payload identity, child/listener drain and closed unknown-SQL/barrier controls. |
| Same persistent fixture command with `--initialization-only --directory build/plan49a-interrupted-initialization` | 3 passed, cleanup empty. Lost initialization acknowledgement stays closed; retry resumes the same fresh identities after authentic physical drain. |
| `python -m scripts.tests.canonical_recovery_check target/debug/pse-canonical-recovery --directory build/plan49a-recovery-wrapper --profile-state <functional-profile>`, exclusive linked mode | Passed. Actual source/lineage/product/payload admission, large staging, SIGKILL/reopen, exact protected history, lower/original kernel caps, closed backup/restore validation, stale authority/creation refusal and fresh creation after restore. |
| `python -m scripts.tests.surreal_fixture_check --evidence build/plan49a-supervisor-thaw`, exclusive linked mode; separate `--mcp-only --evidence build/plan49a-mcp-current` | Both passed; `passed.json` records retirement and drained contexts/scopes. Lifecycle includes authenticated native readiness, unknown readback, finite managed worker, launcher death, qualified restart and offline restore. MCP preserves VIEWER write denial and exact context/authentication. |
| `just native-test -p pse-runtime --lib -E 'test(workflow::durable_tests::) \| test(workflow::caller_lifetime_tests::) \| test(workflow::study_tests::canonical_) \| test(periodic_recovery_preserves_local_cancelled_owner_until_turn_release)'`, exclusive linked mode | 21 run: 18 passed, 3 failed; the exact three plus two typed-failure controls passed in a five-test single-thread rerun. Runs `fb4df346-1aec-496f-9c68-9f35c0d82d76` and `dcfa2fe9-f366-4582-bf2d-af6c84025aa6`. Worker loss stays unknown unless refreshed exact fenced proof establishes absence. |
| `just native-test --managed-primary-route -p pse-runtime --lib -E 'test(managed_primary_study_enters_sixteen_) \| test(managed_primary_study_cancellation_) \| test(managed_primary_publication_serves_exact_partial_history_)'`, reference profile, one nextest thread | 3/3 passed, run `3659a6ed-34ab-41a2-a388-c152f4087d93`. All sixteen native owners, cancellation/tail refusal, exact partial history and escaped Arrow retirement; original 160 GiB reference partition retained. |
| `just worker-test -E 'test(worker_runs_authored_case_end_to_end) \| test(study_parallel_workers_admit_one_summary_and_exact_point_attempts) \| test(durable_submission_survives_submitting_process_exit)'`, exclusive linked mode, one nextest thread | 3/3 passed, run `a60d25f3-0d9b-496b-a460-57a9ac322122`. Two independent workers, exactly one summary/point attempt and durable continuation after submitting process exits before worker start. |
| `just py-sync-native-correctness`; `just native-python build/plan49a-python-private python/pse/tests/test_canonical_results.py python/pse/tests/test_studies.py -k 'canonical or durable_study' -n 2 --basetemp build/plan49a-python-private-tmp` | Extension build passed. Python 11 selected: 7 passed, 4 failed; exact affected four passed with `-k 'constructor_preserves or idempotent_cache_clear or admitted_solve or selection_and_progress_reopen'` in `build/plan49a-python-phase`. Original native solution, retained scalar, protected selection/reader and durable study assertions preserved. |
| `python -m pse.tests.canonical_signal_control` result-pull and `--analysis`, exclusive linked current extension/private server | Both passed; `build/plan49a-signals/pull.json` and `build/plan49a-signal-analysis/passed.json`. Actual paused owned response, original KeyboardInterrupt/cause, grant/response drain and subsequent analysis recovery. |
| `python -m unittest scripts.tests.test_surreal_server scripts.tests.test_canonical_recovery_check scripts.tests.test_surrealkit_maintenance scripts.tests.test_test_resources`, light tooling units | 349 passed. Later lint-only server fixture edits: affected two passed. Selector preparing-runner controls: 4 passed. Source/tool/maintenance/type fixture repairs: 55 passed. These counts overlap other tooling selections and are not a grand total. |

`just unit-package pse-operations 'test(typed_refusal_unit) | test(canonical_codec_unit)'
--success-output immediate`, compile24GiB/two jobs, passed **8/8 with zero warnings**;
run `354d849c-eb3c-4af0-8165-2d8ea5550819`. An added assertion initially failed (7/8)
because the first boxed representation hid the native concrete error behind its box in
standard source traversal. The corrected representation retains the original native
error type, Internal kind and exact message for all six MX classes, plus all registry
refusal/codec controls. No warning suppression or policy relaxation was introduced.

The continuing `just hygiene --live` bundle ran under the wide allocation with linked
native capabilities and two compiler jobs; its actual result was **22 passed, 6 failed**
leaves against zero, at
`build/assessment/20261021T153304.715241Z-hygiene-2075353-3327ad/summary.md`.
Generated equivalence (all targets plus Python annotations/actual installed stubs/Ipopt),
family pins, Rust documentation, boundaries and the other leaves passed. The six failures
were typos, patch-file license metadata, Ruff, Pyrefly and both Clippy modes. The first
four passed their affected recipe reruns with zero findings; unrelated historical resources
remain retained when the preserving cleanup owner cannot operate their retired profile.
Clippy's large-error cascade (574 library/587 library-test diagnostics per mode) is repaired
by boxing the original native store error, without suppressing warnings or losing its cause;
redundant fixture defaults, cfgs and item order are corrected. A second two-mode Clippy run exposed one remaining
test-only Option borrow expression (one error per mode); its correction and both final
default/no-default modes passed with zero warnings/errors at
`build/assessment/20261021T154703.561727Z-clippy-2168316-d9773e/summary.md`.
Full linked-native/canonical/maintenance workspace/all-target Clippy also passed with
`--features pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-operations/canonical-maintenance`
and explicit force-validation, `-D warnings`, wide allocation/two jobs;
`build/qualification/plan49a-final-static/native-clippy.log` has zero warnings/errors.
`just lint-native-contracts lint-native-data governance features-powerset docs-rust`
then passed under the same wide/two-job linked environment; raw output is
`build/qualification/plan49a-final-static/manual.log`. Governance's 25/25 tests, all
generated equivalence/family checks, **419/419 depth-two feature combinations**, **33/33
no-default package checks** and final-source Rust documentation pass with zero
warnings/errors. The continuing governance and powerset summaries are
`build/assessment/20261021T155109.894585Z-governance-2186097-89c10e/summary.md` and
`build/assessment/20261021T155258.867217Z-features-powerset-2195251-c3393b/summary.md`.
The initial hygiene bundle plus affected successful reruns is the acceptance account;
there was no second monolithic hygiene run after documentation edits.

`just docs` passed for the 586-chapter book and scoped search. Metadata/lifecycle validation
also passed with zero errors. `just ready` passed skills/doctor against the final selected
MX1.0 profile; both selected supervisors were explicitly readmitted while stopped/drained.
`just turn-end` passed ADR indexing/formatting; it is not product qualification.

**Tested, final installed handoff:** `just py-sync-native-correctness` rebuilt and installed
the final linked force-validated extension and passed its actual compiled API stub check.
`python -m scripts.surrealkit install` published the final SDK input artifact
`034f5326d548f218fc16a44c25049ed98ef2456ba2b361c4a47b351f5c5c7dcb`
without modifying earlier producer artifacts; `build/qualification/plan49a-final-static/install.log`.
Both supervisors were explicitly readmitted from final source while stopped/drained.
The first ad hoc handoff assertion mistakenly compared the observation's state path with
the readiness enum; its log already reports native READY. The corrected read-only check
uses `ServiceObservation.phase`, verifies authenticated listener ownership/open admission,
and checks reference/private drain without replaying startup or SQL mutations.
That check passed at `build/qualification/plan49a-final-static/handoff-verified.log`.
The functional instance `45ede5c2-0142-49fc-8b20-3e0a750c1106` is open and authenticated;
the reference instance `9a91dcbb-51e3-4722-b80b-603e55edb81d` and private SIGINT profile
are stopped/drained. Final `just ready` passed skills/doctor, including the current linked
extension and open MX1.0 profile, at
`build/assessment/20261021T172503.076540Z-ready-2531419-afa349/summary.md`.
Final publication after completion/navigation changes also passed: `just docs` published
586 chapters with scoped search (`build/qualification/plan49a-final-static/docs.log`), and
lifecycle validation reports zero errors across 1002 native scope items. Retirement
preflight confirms that the completed companion is still consumed by the active
coordinator, ADR and boundary/maintenance documents; owner release is not established,
so it remains retained. Other Plan49 packages and unresolved combined findings stay open.

### Mistakes corrected

Actual controls exposed optional field assertions skipping NONE, lost typed errors across
gRPC status, session ownership gaps, protobuf over-accounting, a terminal seed-manifest
mutation seam and stale payload-freeze state after acknowledged reopen. Their implementing
repairs preserve native rollback, exact identity and uncertainty; affected controls passed.
Native recovery initially treated all expired point claims alike. It now proves never-started
absence only from an exact post-fence projection; a native-started lost worker remains
unknown and cannot gain a retry. The proof controls are sequential cases, not a forced
concurrent start/recover interleaving.

The old worker recipe bypassed the preparing runner and its declared 16 MiB libtest stack;
three selected tests aborted. The recipe now uses the existing runner and explicit store
resume. The repaired selection passed. Authenticated physical drain of the two aborted
fixtures retained their incomplete SQL dispositions and pins; no failed record was rewritten
as successful. Initial Python tests used removed physical flags and obsolete WS prose;
they now consume the generated phase enum and typed failure classification. The analysis
SIGINT controller watched a retired reservation name and never sent its signal; the fixture
now watches the actual scientific-selection owner and reports controller failure first.
Initial failed fixtures and unknown maintenance outcomes remain retained, closed and pinned.
Earlier placement/loader refusals started no product test; their corrected runs use the
declared native environment and allocations. The first Python launch also failed before
collection because the shared temporary root belonged to another user. Private `--basetemp`
directories corrected that environment issue without changing shared filesystem ownership.

### Deliberate deviations and supported limits

**Implemented:** `ProblemTurns` is retained. Source staging/close and begin-run share the
`source-creators:<problem>` write guard with reclamation; sequence assignment retains its
commit guard. Removing these without a replacement would break source lifetime or paging.
Renewal/cancellation and distinct problems retain their separate scope. Actual sixteen-lane
controls demonstrate progress under these conditions; no contention or throughput win is
claimed. All assessed graph relations carry attributes/receipts, so classic ENFORCED edges
remain; bounded inline dependencies use a high-degree fallback. Native plan observations
separate output limits from engine scans: e.g. 17 emitted after 53 scanned, and fallback
64 emitted after 129 scanned. There is no bounded-scan or speedup claim from those limits.

**Tested limits:** fsync EIO, process death, torn bytes and corrupt headers are named fault
models, not a hardware power-loss experiment. Current synchronous settings are selected;
asynchronous queue controls do not authorize asynchronous durability. No existing-data
preservation is claimed. The engine's 4 GiB encoded-transaction refusal is source-derived;
the narrower application/frame/aggregate limits are exercised without allocating a 4 GiB
test transaction. Asynchronous queue saturation is not tested in the selected synchronous mode.
No all-product/native/scientific-model campaign, new-platform, release/distribution,
exhaustive feature powerset or statistical performance qualification is claimed. Existing
Plan28/32/40 and the remaining Plan49 packages keep their owners. No benchmark is required
to support an unmade quantitative benefit claim. Formal ADR acceptance remains separate.

## Execution checkpoint — remaining series authorized, 2026-10-21

The following development observations retain their original stage and evidence scope. The
[pause handoff](#pause-handoff--requested-closeout-2026-10-21) is the current state; references
to pending checks below describe those earlier stages.

The maintainer authorized the conversational execution approach for all remaining 49b–49h
packages. Clean starting source is `4065690aa`. STO01–STO08 remain done and RES04 remains
superseded; existing generated data can be regenerated. Proposed ADR-0215 and blueprint241
record remaining target contracts before dependent implementation. I24 explicitly retains
the opt-in native-child experiment and retires only the unconsumed relation canonicalizer.

Implemented work awaiting targeted acceptance includes RUN01/RUN02 supervision, RUN07 status
constants/mapping and declared numerical binding, RUN10 retained-worker join, PRE01 retention/
firewalls and PRE03 current-identity indexes. BND01 strict vocabulary projection and TOOL07
canonicalizer retirement are integrated at source; generation and survivor checks remain open.
PRE05 pure policy producers, RUN03 paired outcomes/shared preparation and RUN06/RUN08 native
seam/memory work are in progress. Root owns registry, generated
contracts, decision publication and integration. TOOL03 now invokes the authored Rust
cooperating-source executor through the existing authenticated command-group owner. The Python
protocol/state machine is removed; targeted actual launch, log-failure and drain controls and
public command journeys remain pending.
No historical data backfill or broad qualification is scheduled mid-plan. Remaining package
acceptance, inquiry outcomes and assembled QUAL scope are still open.

The PRE01/PRE03/PRE05 producer checkpoint is **Tested** at the selected pure compiler/model
boundary: `just check-package pse-compiler` (all targets) completed with zero warnings/errors;
`just unit-package pse-compiler 'test(workspace::modeling::reuse_tests) | test(workspace::modeling::granular_tests) | test(executable::projection_tests) | test(executable::policy::tests)' --test-threads 1`
passed 45 tests, zero failures (249 filtered), and the selected `pse-model` prediction-policy
unit tests passed two, zero failures (64 filtered). Baseline is zero. The actual 143-body B2
fixture retains no-edit/unrelated reuse without ordinary rotation; bounded churn/oversize
and dependency/policy controls are included. Runtime RUN05 consumers and assembled qualification
remain open. Subsequent PRE02 declaration/generator work is outside that tested checkpoint.

TOOL01 now has one actual compiled document catalogue, independent serialization/deserialization
schema contexts, direction-dependent output names and generated native registrations. Its
fingerprint includes canonical schemas and owning interpretation versions rather than unrelated
feature bits. Completion now carries Version1. Full `just codegen` publication has passed under the selected functional 64GiB,
single-job allocation, including actual directional document registrations and candidate
Python annotation controls. Wrapper migration, standalone version roots and feature
comparison remain open; publication does not qualify the native/Python boundary.

The PRE02 checkpoint is **Tested** by the source-syntax schema control, 15 expression
traversal/checking units, generated grammar metadata control and 45 compiler reuse/policy
units: 62 passed, zero failures. The producer-only PRE04 checkpoint is **Tested** by
`just check-package pse-modeling` and eight selected `resolved::`/`derived::` units,
including canonical-layout correspondence; zero warnings/errors or test failures.
Finite-fact collection and complete function/provider/indexed/reduction support, direct
lowering and all production consumer replacement remain open.

The consolidated mathematical checkpoint is **Tested** by the selected `pse-math`
`unit-package` run with `compiled-evaluator`: 22 passed, zero failures. It exercises
product preflight/abort, shared value definitions, symbolic mixed-partial composition,
actual C++ alias regions/private worker scratch and existing product guard/contracts.
SelectedPartials First/Second dependent regions consume symbolic local partials; standalone
and reverse/action routes retain their existing realizations. Subsequent per-region profile
and action integration is outside that checkpoint. No assembled product or speed claim follows.

RES01 now declares the four distinct scalar key/sort shapes and dense value selection on
owning registry fields. The new source-catalogue schema and generator controls each passed
one test with zero failures; two initial generator control runs inspected the prior
hydrated registry before publication and failed. The corrected control inspects source assembly.
Runtime field eligibility consumes this generated inventory; actual-session predicate
execution is authored but awaits its focused runtime controls. Exact admitted batch
statistics, pruning, sorting and locator retirement remain open. RUN04 Fit/Simulation point
preparation and bound replay, and RUN05 annotation/view consumers are saved at source;
their focused controls and assembled acceptance remain pending. The subsequent full
`just codegen` run also passed all five targets with these source changes. The current
`just check-package pse-py` completed all targets with explicit force-validation and zero
warnings/errors after correcting the semantic-identity borrow and retaining the original
stored-completion metadata lease through document construction. This is compile evidence;
no installed extension/Python journey is qualified by it. Baseline remains zero.

The latest consolidated mathematical retry again passed 22 tests with zero failures, now
including consumed per-region construction profiles and the independently specified
Jacobian action. Weighted/adjoint eligibility and provider/branch region controls remain open.
The same latest source passed 16 selected `pse-backend-native` controls using explicit
kinsol/pounce/idas/uno features and solver/klu/uno capabilities, three selected linked
`pse-uno-sys` curvature/storage controls, and the `pse-diagnostics` effective-text-budget
control. All 42 tests in that combined queue passed with zero failures. This includes the
actual native freshness/loader, bounded owned UTF8 diagnostics, provider-link parser,
preinitialized-host Pounce serial path, fresh KINSOL maps, FERAL symbolic-peak refusal and
Uno status/stop/storage boundaries. The subsequent runtime libtest compilation completed
with zero warnings/errors under the same 64GiB, single-job allocation. Its selected
22-test queue ran: 19 passed and three failed against the zero baseline. The failures
were two parameter-point fixtures registering a composed neutral unit as atomic and
the inherited-stop horizon fixture omitting its required estimator/controller. Fixture
repairs and their focused retry remain pending; these results do not establish complete
RUN04/RUN05 or assembled runtime acceptance.
The maintainer subsequently requested 16 compiler jobs, concurrent independent builds and
no operating-system memory budget for ordinary builds/tests. That persistent local opt-in
is implemented, retaining process/drain ownership and the separate scientific pool, timing
and reference contracts. Nine focused mode controls and twelve existing allocation controls
passed; two actual kernel build owners overlapped with sixteen compiler jobs, sixteen
physical cores and `memory.max=max`. This establishes resource configuration, not a measured
build-speed improvement. The earlier 64GiB/single-job results above
retain their original conditions. Readiness found host Python patch drift: `.venv` had
followed `/usr/bin/python3.14` to3.14.9. A locked
sync recreated it with managed3.14.8, preserving the repository pin. The editable extension
must still be built before Python execution. The subsequent `just ready` passed with that
missing extension reported as a warning; four unsupported historical service records were
preserved and their cleanup warnings remain. Neither readiness nor resource controls qualify
the uninstalled Python boundary.

The expanded normal `just unit-package pse-math` queue with `compiled-evaluator` and one
test thread passed 191 tests, zero failures (266 filtered), including all five regional
product kinds across actual provider/branch barriers, shared maximum-demand local partials,
reverse construction plans, deep derivative exports, private compiled worker scratch,
compiler cancellation/drain, curvature, implicit products and assembly. The current PRE04
finite-fact/shared-symbol producer passed ten selected modeling units, zero failures; the
earlier lineage-path fixture assertion was corrected before that retry. These are focused
component boundaries; final resolved execution and whole-series qualification remain open.

BND02 now has authored registry contracts for keyed physical starts, initialization stages,
diagnostic samples and tolerances, with one pending-to-admitted integration-controls owner
under construction. BND03/BND04 source work adds explicit external versus qualified-fit
origin, private process-local verification, exact protected fit hydration/reconciliation,
required parameter quantities/units/scales, and prepaid sequential faer PSD admission.
SolveSettings advances to version5; covariance result relations advance to version2.
Full zero-argument `just codegen` subsequently published all five schema targets, Ipopt
bindings and candidate Python annotations. Nested generic documents now retain distinct
owner names/defaults, and identity-keyed maps retain their actual key role. Two targeted
identity-map schema controls passed, zero failures. The native-feature all-target
`just check-package pse-py --features solver-kinsol,solver-diffsol,solver-ipopt,solver-highs,compiled-evaluator`
then passed with explicit force-validation and zero warnings/errors. This establishes
generation and compilation, not native/Python behavior. Full canonical, Python,
resource/cancellation and scientific acceptance for these boundaries remains open.

The first controlled result-file units passed three columnar controls and six relation
controls, zero failures; the latter compilation reported three test-only trivial-cast
warnings, whose repair remains in progress. Two operations allowance/proof controls also
passed, zero failures. The initial columnar test compilation refused an ambiguous empty
schema fixture, repaired before its passing retry. Actual protected-provider pruning,
locator retirement and assembled result qualification remain open.

The current integration checkpoint carries an explicit standalone wire selection for all
144 exported owners: 58 native versions, 85 flat-v1 roots and one value-v1 root. Existing
fixed ProcessCheckpoint4 and InvocationEvidence2 remain their native version owners.
Nested values keep their original domain meaning; explicit body/root joins replace
implicit caller reinterpretation. Submitted starts now use the existing complete native
receipt declaration, with borrowed numeric/IEEE-bit serialization under actual report
custody. The public source-reclamation continuation is part of the same compiled catalogue.
Full generation and the complete native Python compile passed. Twenty-one targeted
native-local controls passed against the zero baseline, including document headers,
parameter-valued simulation IC/trajectory/replay/schedule behavior, LM policy, submitted
receipts and tear coordinate receipts. Actual default and linked-native force-validated
extensions now build and generate their stubs, and their direction-qualified catalogue
fingerprints agree. The installed linked-native extension passes 43 standalone-wire,
private-edit, solve-settings and native-boundary units plus two physical-input units,
against the zero-failure baseline. Initial failures exposed missing identity hooks in
private edit conversion and stale root/body, input/output and optional-name fixture
assumptions; those are corrected. These checks do not qualify every exported root,
every feature combination or the assembled product.

RUN09 implements fixed independent library-relative cost, step and cosine criteria,
effective work caps, FitProfileV6 and ReplayDocument3. Five LM controls cover the actual
original-policy comparison, cosine/stationarity disagreement, qualification/covariance
withholding, limits/cancellation and replay/profile chains. The canonical retained-result
control also passed. This resolves I18's bounded policy inquiry while the package retains
its wider Python/scientific qualification obligations. Proposed ADR-0215 and blueprint242
retain the physical study request8/stored definition12 and standalone wire target.

TOOL04 implements one canonical source cursor and shared physical source deletion, one
bounded foreground page plus explicit continuation, without caller cursor authority.
Cleanup follows the completed mutation timeout owner under the same original deadline;
a deferred page cannot replace its immutable receipt. Actual canonical controls passed
for protected-first-page progress, peer reopening, replay, deadline behavior and exact
root/pin races. The combined nineteen-control selection initially had seventeen passes
and two fixture failures; the corrected pair then passed. One fixture incorrectly reused
a unique guard incarnation; another expected withdrawal to restart the cursor instead
of resuming the six-candidate tail after a protected 64-candidate page. Their corrected
assertions preserve uniqueness and exact continuation. The actual protected-pruning pair
released scalar locator retirement. Its relation, indexes, append/replay/collection
machinery and callers are now deleted with complete generated substrate-v8 outputs.
Fresh v8 state is initialized and authenticated startup passes. The v8 canonical
selection initially passed 21 of 23 controls; both corrected lifetime controls then
passed separately. One now includes the foreground retirement receipt in its exact
one-batch total. The other checks actual charged buffer custody and unchanged scalar
bits through packing, while allowing obsolete source-container metadata to retire.
Selected scope includes provider pruning, historical manifests, retained arrays,
protected reads/exports and append ACK ownership. Native protected-file maintenance
phases and assembled qualification remain open.

Supported initialization exposed ordinary managed command parents that could not hand
work to native children. The command parent now authenticates its exact registered
cgroup, invocation and inode before establishing foreground custody; it gains no native
reuse authority. Seventeen targeted controls passed, including actual-kernel normal and
original-deadline descendant drain. All owned test scopes and allocations were released.
PRE04's direct typed producer, compiler/portable consumers and per-body receiving
source bindings are now integrated in the checkout. Specialization emits typed roots,
original-source correspondence and physical obligations during its one construction rewrite;
compiler projection and numerical construction consume those products directly. The old
dispatch-body expansion and its final consumer are removed. Ordered composition retains
original guards, nominal member boundaries, ranges, actual input contracts and independent
source/member tokens. Provider conversion/partial contracts are prepared before numerical
construction and restored only inside qualified recipe sessions. Broader PRE04 acceptance,
remaining retirement and assembled qualification remain open.

### Pause handoff — requested closeout, 2026-10-21

Verified 2026-10-22. The maintainer requested integration, correction, qualification,
documentation and local commits before pausing. On 2026-10-22 the maintainer canceled the
broad native sweep and selected representative functional assessment without a benchmark
or production-scale campaign. Remaining 49b–49h execution is paused; nothing here completes
those packages or authorizes automatic continuation. There is one checkout on `main`, no
worktree merge is needed, and all workers and one-shot closeout jobs have drained.

**Implemented:** direct typed model/body production and consumers, receiving source maps,
demanded original-equation preparation, protected result/maintenance consumers and strict
directional Python documents are integrated. Original supplier projection now retains real
independent coordinates and only substitutes selected suppliers and their demanded dependency
closure. Private inner KINSOL execution honors explicitly tighter normalized precision while
retaining public defaults, native failure qualification and the original accuracy assertions;
its solver identity advances to v2. Physical body premises remain distinct from the complete
receiving deployment envelope. Generated scientific artifact completeness is qualified by the
admitted modeling/simulation/fit family. Positive native fixtures admit their complete finite
construction envelopes, and escaped registrations prove actual program ownership after cache
clear. No compatibility path or scientific tolerance relaxation was introduced for these fixes.

**Tested, baseline zero:** the final completed checks below passed. Rust correctness selections
used explicit Arrow force-validation; Python used the rebuilt linked, validated extension,
Python 3.14.8 and 16 workers. Ordinary build placement retains the maintainer's unbounded-memory,
16-compiler-job selection; scientific children retain their own finite contracts.

| Command / scope | Result and boundary |
| --- | --- |
| `just clippy` | Full workspace/all-target default and no-default graphs; zero errors/warnings, keep-going, warnings denied |
| Native `just clippy-default` | Full workspace/all-target selected native/canonical/maintenance graph; zero errors/warnings; feature selection below |
| `just typecheck`; `just lint-py` | Pyrefly zero diagnostics (three existing suppressions); Ruff zero findings |
| Full zero-argument `just codegen` | All five publication targets passed, no final generated drift |
| `just test -E '<functional-filter>'` | 3,491 passed, zero failures, nine skipped; default graph on the new functional store, run `e4710ccd-8c7b-42ba-8138-4d00a5801840` |
| `just doctest` | 48 passed, zero failures or ignored doctests |
| `python -m scripts.python_tests --maxfail=0 --continue-on-collection-errors --basetemp <owned-directory>` | All 270 unit/component tests passed in 147.44 s; integration/performance/parity excluded |
| `python -m scripts.setup_report <owned-output>` | All 1,241 tooling tests passed, zero failures/errors/skips |
| Selected native-local repair controls | 17 backend precision/policy, 50 runtime, 19 child/target, 13 IBEX, six dynamics and four Pounce controls passed; these selections are not a unique combined test count or full native qualification |
| `just ready` | Skill synchronization and doctor passed against the new 3.3.4/MX 1.0 deployment |

The functional filter is `not binary(=worker) & not (package(pse-tests-conformance) & test(/^acceptance::/))`.
The final native Clippy feature selection was
`pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-operations/canonical-tests,pse-operations/canonical-maintenance,pse-tests-conformance/native-acceptance,pse-benches/native-process,xtask/canonical-tests`.
The linked extension and its actual compiled API stubs were rebuilt before Python execution.
The child/target selection passed 19/19 with a frozen terminal, completed projection and released
host allocation; run `e3aa395b-6a23-4f2a-a753-28a51183a406`.

**Canceled, not full native qualification:**
`just native-test --profile local -E '<functional-filter>' --features pse-operations/canonical-maintenance`
selected 4,779 tests across 80 binaries. The maintainer interrupted it after 862.203 s of test
execution: **2,759 passed, 11 SIGINT, 2,009 not_run**, with 30 tests and one binary skipped
(including six managed-primary tests excluded by the default filter). No failure was reported
before interruption. Nextest labels the interrupted cases failures; they are not 11 diagnosed
regressions. Run `16c32128-60a0-4317-955e-c3b37af45bd2` and
`build/native-tests/1792650816881675357/native.json` retain the selected scope. The frozen
invocation is `f8d0dc8ea0e2480f849e420e4517e61b`; its XML preserves individual outcomes and
durations. Its assessment is incomplete, while terminal acknowledgment, metadata projection,
physical process/scope drain and host allocation release completed. The identity-mismatch
footer reports interrupted coverage, not additional scientific failures.

Individual completed results remain bounded evidence. In that interrupted run, the repaired
exact-protection settlement, bounded interval reclamation and blocked-product controls passed
in 11.789 s, 14.260 s and 24.136 s respectively. These durations include concurrent execution and
fixture work, not isolated mechanism measurements. Full integrated worker/acceptance/managed
journeys, parity, performance, feature powersets and platform qualification were not_run.
No build-speed, scientific-speed or memory-capacity improvement is claimed.

**Functional assessment:** the broad sweep mixes pure/native-local controls with expensive
live-store qualification. Each initialized UUID fixture installs 251,529 bytes containing 777
table/field/index/function/event declarations in one atomic transaction, with two physical
clients, authorization/context selection and durable registry registration/drain. This is
per-fixture work, not 777 external RPCs or per-query Python mediation. Sampled concurrency
varied from one exclusive control to 16 simultaneous store tests; low occupancy was not a
compiler-job limit. More importantly, two four-byte artifact cases explicitly wait for the
production 60-second original creation window. Whole-test timings do not isolate schema,
transport, registry or cleanup cost. The [representative functional route](49h-qualification-and-design-publication.md#representative-functional-checks)
owns the revised qualification approach and proposed fixture corrections. No replacement
campaign, benchmark or scale/setup rewrite was run after cancellation.

**Store and retained-state handoff:** earlier ENOSPC poisoned the old functional engine.
Free space did not clear its fail-stop. Normal restart correctly refused three unobserved
fixture-creation borrowers from earlier invocation `d63c4731f12e4cc7b2a0d7eef19f7fac`.
That old service, data and evidence were preserved. Its dead-job owner
`bb40005600a842fab3327793beb5dd15` still retains one build slot through its unsatisfied
terminal guard; no supported same-boot recovery acknowledgment was fabricated.

A separate generated deployment was initialized through the normal canonical-tools route and
opened with the current `pse.substrate.v8` contract. The ignored local selector now names
`~/.local/state/pse-arrow/surreal-plan49-closeout-20261022`, gRPC loopback18349,
instance `6e982d05-8e9a-452f-a4a6-c39af9104387`. Host policy explicitly admits two functional
stores, each retaining its own 8 GiB charge; all 57 host-admission controls passed, including
third-store refusal, timing isolation and retention of an unknown owner. Ordinary build
memory policy is unchanged. Both persistent services remain intentional resident state.

The newly canceled campaign retains 217 pinned resources: 206 recorded physically drained,
11 with unobserved creation effects despite dead borrower PIDs. Its current finalizer marks
all resources incomplete when overall selected coverage is interrupted. None was deleted
or relabeled successful. These retained effects differ from the fully drained job processes.
TOOL04/RUN01 retain the recovery/disposition obligation, including truthful per-test outcomes;
unknown external effects cannot be released merely because a process disappeared. Unsupported
historical profile records were also preserved rather than rewritten to silence cleanup output.

**Next work, only after explicit resumption:** implement the representative functional fixture
and selection changes in 49h, retaining actual contract semantics and all refusal/source/clock
assertions. Address retained-state disposition through its authenticated owners. Reconcile
remaining producer/caller retirement and package acceptance, then select the still-required
assembled QUAL journeys and enduring publication. GOV02, the remaining 49b–49h packages and
whole-series qualification remain open. Proposed ADR-0215 does not by itself establish them.

### Bounded native inquiry dispositions

I14 adopts the one-thread Pounce caller path and retains its admitted local pool for
larger batches. The actual native control passed with an already initialized host global
pool. Symbolica3.0.1's reached unlicensed `LicenseManager::new` still attempts a one-thread
Rayon global initialization and ignores failure; a preexisting host pool stays unchanged.
Valid license/unlock paths bypass that fallback. This is distinct from evaluator construction:
`LicenseManager::max_threads` bounds the explicit requested cores, and the optimizer uses
scoped threads with the caller executing the final lane. The current production default is
one requested core; scalar interpreted/C++ evaluation is sequential. Retain complete foreign/
process envelopes and add no eager project global initialization. No blanket elimination,
resizing or ownership of an embedding host's global pool is claimed. The 22 math controls
exercise the configured environment; fresh-process unlicensed/embedding initialization and
constructor-team/cleanup qualification remain required before claiming those combinations.

I15 retains the complete foreign allocation envelope. Tagged [PETSc3.24.0 allocator source](https://github.com/petsc/petsc/blob/v3.24.0/src/sys/memory/mal.c)
requires installation before initialization and does not supply original allocation ownership
on later free/realloc. The deployed SuiteSparse_config7.7.0/KLU allocator setters are global
function pointers; they neither establish lane/owner custody nor cover SUNDIALS/BLAS/MKL
allocations. No complete original-owner/size protocol and bypass coverage is established,
so hooks are not adopted. Existing PETSc process initialization ownership remains intact.

I17 selects independently fresh KINSOL map sessions. Tagged [KINSOL7.1.1 source](https://github.com/LLNL/sundials/blob/v7.1.1/src/kinsol/kinsol.c)
shows vector allocation, callback resets, counter initialization and linear setup, but does
not establish complete fresh-equivalent Anderson/map/KLU history under a neutral retained
reset. The authored map execution therefore creates a fresh session per application. The selected
native fixed-point/Picard controls have passed; runtime/public assembled integration remains open.

I23 retains the declared Uno HiGHS SQP-LBFGS integration documented in
[the numerical binding note](../../docker/solvers/uno-binding.md). The four numerical hunks
remain explicit and distinct from integration/determinism work. Source comparison with
upstream2.9.1 finds a BQPD operator-Hessian candidate, but equivalent controls, accuracy,
acquisition, storage allowance and readback are not qualified. IPM or unconstrained-only
algebra is not a substitute for the selected bounded SQP method. No upstream/dependency
change is claimed. The candidate becomes actionable when those exact obligations can be met.

TOOL02's versioned Rust campaign/settings/run documents now own the defaults, strict decoding,
name inventory and package-relative source/diagnostic checks. Python uses generated msgspec
TOML types and the common Rust context admission; the Rust reference derivative controls use
the same manifest owner. The reference manifest now explicitly supplies interpretation1.
Generation has passed; selected Rust/Python conformance controls remain pending.

RES05 selects removal of the unreached file-cache adapter after current producer/caller
inspection: `bind_state*`, source-generation namespaces and metadata/statistics/listing
activation have no production caller. Their policy knobs, detailed reports and dedicated
mechanism tests are removed with the adapter. Actual syntax/model/preparation caches and
load/query/output admission remain; DataFusion file-cache capacities remain explicitly zero.
The reached physical-plan predicate allowance remains independently owned. Survivor tests
and boundary generation are pending; no hit-rate or latency improvement is claimed.

MAT01/MAT02's selected compiled-evaluator controls are **Tested**: the focused `pse-math`
unit command selecting product preflight, constructor abort, scalar definition-map admission,
deep shared lets and sibling regions passed ten tests with zero failures against baseline zero
at 64 GiB/jobs1. Final resolved producer identity, higher derivative cutover and full math/
scientific acceptance remain open; these tests do not close either package as a whole.

TOOL05 has moved the exact protected result-selection/predicate/usability contracts to the
workflow result owner. Offline pack tooling consumes those types; full/selected result
readers no longer obtain their scientific selection contract from an export module. The
internal tooling extraction is now authored: live detail collection and shared collection
budget/error contracts belong to `observation_detail`; capture, inspection, comparison,
review analysis, packing and publication composition belong to `tooling`. The workflow
owns protected selection contracts, and tooling calls its bounded reader interfaces.
Rust/Python callers have moved; the old root module exports are removed. Compilation and
the affected caller controls remain open.

TOOL04's guarded domain-mutation source is authored. The registry declares maintenance
generation/page-effect records, and generated functions own authority rotation, placement
rehoming, validation-read admission and the existing exact schema-activation transaction.
Supervisor code retains authentication, physical copy/drain and phase orchestration. Page
effects retain original inputs and acknowledgements; a generation change fences older
callers. The script checks persisted phase/original-operation identity before requesting
the database generation. Generated-function behavior, interruption/replay, stale-generation
and protected-payload controls remain open; source publication alone establishes none of them.

The next shared generation checkpoint includes Study operation/request8 and stored
definition12, physical Arrow boundary tables and role vocabularies, covariance relation2,
and optional controlled result statistics. Generic fit/integration schema names now
distinguish concrete instantiations; catalogue-derived names apply consistently to roots,
nested definitions and references. Identity-keyed map schemas retain declared key roles
rather than inferring identity from hexadecimal patterns. The first attempted generation
stopped on the generic-name collision; the repair and subsequent source joins are being
regenerated before their runtime/Python controls. No full package closure is claimed.

### Observation inquiry disposition

The TOOL06 inquiry is **Tested** in the selected native runtime queue: both
`observation_layer_inquiry_actual_steady_and_fit_preserve_science_but_expose_phase_gap`
and `observation_layer_inquiry_bounds_state_and_retires_on_future_drop_and_panic` passed.
Their original science agrees across all three observation modes, while the missing native
phase prevents replacement of the actual observation owner. The finite candidate holds at
most 64 active spans and 256 closed samples and retires them on joined completion, future
drop and unwind. This is a semantic/ownership no-adoption outcome, without a measured CPU
or memory benefit. The optional crate remains unselected: current boundary-catalogue and
private runtime consumers need a deliberate dependency seam before a separate crate could
provide independent build value. The corrected internal tooling module remains selected.
These two passes do not close the separate TOOL05 consumer or whole-series qualification.

### Symbolica optional inquiry dispositions

I11 retains canonical scalar formals and selected argument-free `FunctionMap` aliases.
Pinned Symbolica3.0.1 supports function-shaped indeterminates; capability absence is not the
reason. Reached dependency, guard, class and factorable consumers currently key roles by
`Symbol`, so a common `slot(k)` function would collapse distinct slots until those consumers
admit exact indeterminate keys. Revisit when that role/check migration removes a measured
registration or retained-symbol burden and fresh-process reversed-registration, derivative,
selected-pair and C++ export controls preserve identities and domains. Current alias sharing
remains selected. This is a source-based no-adoption decision, not a failed benchmark.

I12 retains the interpreted/C++ scalar profiles. The [official evaluation guide](https://symbolica.io/docs/numerical_evaluation.html)
documents JIT, vectorized evaluation and generated SIMD/CUDA libraries. Exact pinned
`evaluate/backend.rs` and `evaluate/export.rs`, plus SymJIT2.26.0 sources, determine the
remaining integration. A JIT candidate must explicitly disable fast math and library threads,
bound stack/code/private scratch, and qualify primitive floating semantics and construction
cancellation; lack of contraction controls is not asserted. SIMD needs admitted pack/tail
buffers, fixed ISA and lane/error ordering. CUDA needs host/device/compiler owners and finite
chunk cancellation through synchronization/drain. Current region consumers do not supply
those owners. The existing test-only `product_realization_probe.rs` already invokes a
thread-disabled, fast-math-disabled JIT candidate on guard-free polynomial rows and reports
ISA conditions; this turn has not executed that probe or measured its benefit. Its existence
does not establish a production profile. Revisit an actually grouped repeated region when a total preparation/evaluation
comparison includes these costs. No device or speed benefit is claimed.

I13 retains the reached library algebra: `coefficients.rs`, `implicit_cubic.rs` and
`implicit_selection.rs` already use Symbolica polynomial/coefficient/root operations with
consumed degree, finite-value, domain and original-residual controls. A broader rational
reformulation must preserve excluded denominator points and parameter incidence. Generic
[solution sets](https://symbolica.io/docs/solving.html) require explicit domain, coverage,
coverage guards and branch predicates; unsupported/incomplete cannot become empty or a
selected numerical root. Revisit an actual selected-function certificate or repeated
factorable transformation burden with those obligations. A transformer wrapper alone does
not remove an existing project rewrite engine. [Term streaming](https://symbolica.io/docs/streaming.html)
is not selected for dependency-sharing work: pinned `streaming.rs` would introduce its own
pool, disk/tempfile and teardown/cancellation owners. Revisit only a reached finite term-local
spill whose complete resource and cleanup contract is qualified.

I25 does not add hot-start history. Current `exact_evaluator` uses direct translation;
pinned `evaluate/tree.rs` refuses a supplied hot-start scheme with positive Horner iterations,
while zero iterations only linearizes. The accepted input is an explicit prior expression
scheme, not a cached evaluator. Equal expressions/settings/definition maps already reuse
`ScalarProducts`. Revisit a reached non-direct changed-body caller only with the completed
scheme in versioned identity/cache/serialization premises, cancelled-construction refusal,
derivative/guard equivalence and a total preparation comparison. No extra `optimize_stack`
pass is added: the selected library construction already invokes it; an actual temporary
census must justify another pass.
