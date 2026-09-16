---
title: Unified DataFusion and Delta Lake simulator — hard pivot
status: in-progress
date: 2026-09-15
adrs: [ADR-0065, ADR-0066, ADR-0068]
phase: 1
---

# Unified DataFusion and Delta Lake simulator — hard pivot

**Scope changed on 2026-09-16:** the maintainer requested an architecture-only pivot
of existing functions, with schema engineering first and no additional simulator
implementation. [Plan 08](08-schema-first-native-data-pivot.md) is the proposed
replacement execution sequence for that scope. This document retains the broader
functional inventory and historical implementation receipts; its unfinished simulator
packages are not the current work queue. The assessment below describes the Plan 07
boundary before that re-scoping and does not claim those packages are completed.

**Completion assessed on 2026-09-16:** 0 of 13 packages are closed. UD00–UD05 and
UD07 are **Implemented in part / in progress**; UD06 and UD08–UD12 remain
**Proposed / open**. No milestone or design gate is complete. UD07 now includes an
actual native Ipopt operator with bounded execution tests, but no source-derived
problem-to-solve-to-publication product path. ADR-0068 and blueprint revision 39
record the authorized target before related implementation changes.

The [completion review](../design_review/reviews/design_review_unified-datafusion-delta-completion_2026-09-16.md)
is the current evidence assessment: package matrix (§1), live deletion ledger and
provider-level gaps (§4), independent gates (§6), original F01–F14 traceability (§7),
V01–V13 evidence gaps (§9) and prioritized functional cuts (§11). It records **Revise**:
G1 fails for the required single product lifecycle; G2–G7 remain unresolved.

**Immediate remaining cut:** connect the reusable compiler transforms and all opening
callers to exact Delta publications and native plans; delete the custom store,
snapshot/manifest, stage/context/sidecar and artifact-memo lifecycle. The current
compiler and Python handles still use that predecessor architecture. UD03's deletion
exit is unmet, not deferred to UD12. Complete common policy/configuration admission,
candidate completeness and composed member-write recovery as part of that cut.
Then finish native process/problem construction (UD04–UD06), product solver/backends
and durable results (UD07–UD08), change/retention/inspection (UD09–UD10), and full
qualification/deletion closure (UD11–UD12). No compatibility period is introduced.

**This replaces Plan 06's execution sequence.** Plans 03–06 and their receipts are
historical inputs. Their object contracts, unfinished packets and acceptance harnesses
are not implementation obligations. Implementation follows the dependencies below.

The governing input is the
[unified DataFusion/Delta review](../design_review/reviews/design_review_unified-datafusion-delta_2026-09-15.md).
Its F01–F14 findings, V01–V13 checks and G1–G7 gates are mapped below. The listed ADRs
are existing inputs; UD00 records the broader target and supersedes affected decisions
before implementation changes. This plan does not silently amend the blueprint or
mark proposed records accepted.

## Context

### Controlling objectives

1. All product data access, transformation, validation, compilation, execution,
   persistence and inspection use the native DataFusion provider/planner/execution
   framework. Delta Lake owns durable tabular state and table transactions.
2. Deliver the functions of a process-model simulator: authored model and case →
   equations and problem → initialization and solve → explainable, durable results
   that can be reopened and queried.

Everything else is subordinate. Policies, APIs, crate boundaries, graph objects,
stores, hashes, pass choreography and fixtures may change. Reuse code only where it
serves a target function more directly than replacement. Equivalence to the current
implementation is not an acceptance criterion.

### Delivery rules

- One implementation stream builds the target. Packages organize work; they are not
  releases, deployment stages or transition periods.
- No dual writes, compatibility readers, migration campaign, old execution fallback,
  shadow implementation, old/new feature flag or predecessor-equivalence campaign.
- Delete a replaced mechanism and its callers in the same functional cut. Temporary
  compilation failures during that cut are preferable to compatibility glue. Do not
  repair obsolete machinery merely to make its old tests green first. The completed
  target still has a zero-failure quality baseline.
- Existing development stores and runtime objects are disposable. Preserve unrelated
  dirty work, but retain no old runtime schema, graph or data object merely to support
  historical state. Git history and historical documentation are sufficient records.
- New target versions are retained only for live publications, runs, readers or an
  explicit product retention choice. There is no blanket historical archive obligation.
- Use native capabilities before adding PSE behavior. Express required domain logic
  through native expressions, UDFs, table functions or logical/physical extensions.
  Do not build another generic operation interpreter, policy engine or scheduler.
- Reach an end-to-end path early and extend that same path into the simulator. Do not
  finish an abstract provider platform before exercising a process model.
- Resolve library gaps with focused exact-pin tests and, where necessary, a small
  qualified library integration/patch. Do not restore the old architecture or wait
  indefinitely for an upstream release.

### Current evidence

**Interface-checked:** the review inspected DataFusion 55.1.0, Arrow/Parquet 59.3.0,
object_store 0.13.2, Delta development commit
`58f07cd62bfbce3649a7e1c87c696288068ae184`, and kernel commit
`8ba063f8f84fec222000f66d40d70911d7c79675`. Its
[source/metadata evidence](../design_review/evidence/unified-datafusion-delta-context-2026-09-15.json)
records source hashes and the research feature envelope. Combined PSE/Delta builds
and bounded native/Delta behavior have since been demonstrated; the implementation
receipts and completion review give their exact scope. They do not establish the
complete target simulator or final whole-workspace health.

**Implemented in part / bounded Tested:** numerical expression/Jacobian preparation
and a real native Ipopt logical/physical operator now exist. The latest pinned-container
solver run passes 12 tests; the common catalog library run passes 113, both with 0
failures/skips and baseline 0. Exact commands and conditions appear in the latest
receipt below. `pse-structural`, `pse-plans`, `pse-backend-nl` and
`pse-backend-pyomo` remain declared stub boundaries. Product runtime installation of
the solver, source-derived case construction and durable run publication are open.

The current environment diagnostic reports **1 environment-freshness failure,
baseline 0**, and a stale editable extension. Refresh before current Python
qualification. Earlier compiler, Clippy, Python and performance receipts describe
their recorded source state only; none closes a package or architecture gate.

### Required functional outcomes

Blueprint §§7–19 and §25 supply domain vocabulary and the Slice A functional
inventory. They do not require preserving the old implementation sequence.

| ID | Required outcome | Completion condition |
|---|---|---|
| S01 | Author and change models/cases | Packages, sources, declarations, parameters, observations and case targets become typed facts; edits/renames have correct identity and source diagnostics |
| S02 | Construct a process model | Domains, features, topology, ports, property demands, method choices, contributions and balances derive from declarations |
| S03 | Preserve mathematical meaning | Quantity/basis/reference/shape, ordered operands, indexed operators, guards, implicit relations and derivative/kernel requirements survive appropriate lowering |
| S04 | Produce a solvable problem | Index expansion, case overlays, bounds/fixed status, objectives, incidence, DOF, structural diagnostics, scaling and initialization are explicit native operations |
| S05 | Run native simulation | Actual residual/Jacobian evaluation and Ipopt invocation return values, residuals, status and structured failures through native execution |
| S06 | Generate backend routes | Pyomo construction and NL/SOL export/import derive from the same problem contract; invocations and result ingestion use native operation boundaries |
| S07 | Publish and reopen | Delta model/case/run publications select coherent exact table versions; cold processes need no predecessor graphs or custom store files |
| S08 | Change, reuse and maintain | Semantic changes invalidate affected work; qualified exact reuse/CDF and cleanup preserve live model/run/reader requirements |
| S09 | Inspect and integrate | Rust, SQL and Python/Arrow expose the same admitted facts, metadata, diagnostics and outcomes with owned streams |
| S10 | Extend through contracts | A method, invariant, relation or provider adds one semantic declaration plus genuinely new implementation/tests, without new dispatch/storage/inspection interpretations |
| S11 | Handle failure and resources | Invalid, unsupported, incomplete, numerical failure, cancellation, exhausted resources and uncertain commits remain distinguishable |

**Minimum simulator coverage: Slice A.** Include units/material/reference data, ideal
EOS, declared NIST/RPP/Perry's property-method families, FTPx/FcTP states, lumped
control volumes, heater, feed/product, mixer, state junction, equality connections,
nominal scaling and the required structural/initialization behavior. UD00 identifies
the exact declaration IDs and independent oracles; one successful constant-property
fixture cannot stand in for an entire required family.

Native Ipopt, generated Pyomo and NL/SOL routes are in scope, consistent with this
functional inventory. The indexed/derivative architecture also gets a small analytic
discretization example. Completing every later-phase unit model, electrolyte/EOS
family, cloud service or distributed backend is not a prerequisite for this pivot.
Those are future functional extensions in the same framework, explicitly unavailable
until implemented. They cannot be used to defer S01–S11.

## Decisions

### Target basis

| Concern | Implementation decision |
|---|---|
| Authority | One typed declaration system generates Arrow/Delta schemas, field annotations, invariant plans and mechanical bindings. Persisted contract views are not a second editable registry |
| Data operations | Native `LogicalPlan`/`Expr`, operators and functions first; domain UDFs/native extensions where necessary. No restricted parallel execution algebra |
| Hierarchy | Native catalog list/catalog/schema/table providers bind exact invocation inputs. Native memory registries suffice where no additional behavior is required |
| Execution | Shared runtime/resource foundation and composed Delta/PSE planner; invocation state binds exact providers and effective policies. No internal default-session fallback |
| Durability | Delta tables hold durable source/model/case/compiled/runtime/provenance/publication facts. Arrow streams/buffers serve execution; not every intermediate needs a commit |
| Publication | A small typed Delta control table selects exact member versions/revision slices. Delta owns table commits; PSE defines coherent model visibility |
| Mutation | One validating Delta builder/command route with actual session and commit properties. Provider/SQL hooks forward to it or reject explicitly |
| Mathematics | Typed symbolic relations and native transformations; derived graph/sparse/evaluation layouts exist only for actual algorithms and never own model facts |
| Numerical work | Native physical operators own structural algorithms, evaluation, initialization and backend invocation; specialized libraries operate inside those contracts |
| Python | Coarse owned Arrow data/streams and native operation handles; mechanical interface and generated backend adapter, not a second semantic engine |
| Identity/history | Distinguish entity ID, publication ID, Delta version and location; keep hashing only for a target need. No predecessor compatibility; target retention follows product needs |

### Pinned-library integration decisions

Use the [provider map](../capability-maps/datafusion_provider_contracts.md) and the
DataFusion/Delta skills to find native capabilities. Exact Delta skill source takes
precedence over published-release examples; do not use Context7 for Delta here.
DataFusion's skill/pinned source are primary, with Context7 available for discovery.

Before adding data-work helpers, inspect native operators, nested/higher-order
functions, provider hooks, configuration, table properties and builders. Record only
the missing domain meaning that justifies a new extension; no exhaustive research
campaign or closed function allowlist is needed.

The review's critical conditions become UD01 integration tests:

- Pin Delta by full SHA and retain its separately resolved kernel SHA in the lock;
  verify the combined feature graph and one Arrow/DataFusion/object_store family.
- Compose public `DeltaExtensionPlanner` and PSE extensions in normal physical
  planning. Supply an actual `SessionState` to every Delta builder, using
  `RequireSessionState` where offered; setting that policy without a session is insufficient.
- Characterize direct insertion versus the higher-level validating write route,
  including CHECK/nullability and application commit properties. Select one complete
  route; do not retain a permissive provider-hook back door.
- Public Delta write preparation is incomplete as a composition API. Preserve the
  native input plan/dependencies/effects and observe nested plans through the shared
  planner. Expose a minimal prepare/execute seam in a pinned library patch only if
  required; do not copy Delta's private planner into PSE or wait for upstream merge.
- Generic logical-codec arms are unimplemented and the physical codec lacks current
  scan coverage. Encode supported representations, reconstruct exact providers and
  re-plan; refuse unsupported arms before calling them.
- Generate lossless durable forms for unsigned ranges, fixed widths/shapes,
  dictionary identity, timestamps and semantic metadata. Schema conversion is not
  proof of value/meaning preservation.
- Qualify conditional publication, application-transaction reconciliation and
  post-commit failure. Neither a row count nor an error proves commit or rollback.

### Native policy placement and coverage

This is a coverage map, not a requirement to build one custom wrapper per row.

| Native level | Standardized responsibility | Packages |
|---|---|---|
| Runtime / object stores | Shared memory/spill/cache/task resources, storage binding and backend commit capabilities | UD01, UD03, UD07, UD09 |
| Session / configuration / functions | Typed effective settings, actual function bindings and analyzer/optimizer/planner composition | UD01–UD02 |
| Catalog list / catalog | Native registration/replacement, workspace/publication selection and immutable invocation bindings | UD02–UD03 |
| Schema / async resolution | Names, discovery, owner/type metadata, full/subset coverage, absence/error semantics and refresh | UD02, UD10 |
| Table / source / view | Schema/defaults, established constraints, exact snapshots, scans/mutations and view-inlining obligations | UD02–UD03 |
| Factories / table functions | Native opening and parameterized providers/plans; heavy work belongs to execution | UD02, UD04, UD10 |
| Fields / expressions / UDFs | Types, coercion, nullability, volatility, field meaning and truthful optimizer properties | UD02, UD04–UD07 |
| Analyzer / logical optimizer | Domain checks, dependency binding, valid transformations and admission before effects | UD01–UD06 |
| Query / extension / physical planning | Composable native child graphs, declared effects and honest physical properties | UD01, UD03, UD06–UD08 |
| Scan / datasource / format / sink | Native pruning, projection/filter/limit semantics, streams and schema adaptation | UD02–UD03, UD10 |
| Delta table / transaction | Schema/features/CHECK, validated writes, conflicts, actual versions and outcomes | UD03, UD09 |
| Streams / FFI | Backpressure, cancellation, ownership, external work and typed results | UD01, UD07–UD08, UD10 |
| Metadata / EXPLAIN / metrics | Actual bound implementation and queryable domain facts without hidden effects | UD02, UD10–UD11 |
| Serialization | Supported representations, exact runtime reconstruction and explicit refusal | UD10 |

Native settings retain their native definitions. Only domain scope composition,
invariants and effects require PSE policy data. Registration is not validation;
optimizer constraints are not validators; SQL options do not cover programmatic
execution. Address those exact gaps once at the common native boundary, including
views and plans without table scans.

### Required publication behavior

UD01/UD03 settle the exact native operation construction, with this observable contract:

1. Resolve the base publication once; assign explicit candidate/attempt identities.
   Shared tables use revision selectors so another attempt's rows cannot enter this model.
2. Write candidates through the common Delta route and retain actual committed versions.
   Unchanged members may reuse their exact published version/slice.
3. Check the complete exact selection, including keys, relationships and process semantics.
4. Atomically record the publication and conditionally advance its named head from the
   expected parent in **one Delta control-table transaction**. The typed member vector
   and head transition cannot be separately committed facts.
5. Readers pin that root version and exact members; never assemble independently latest tables.
6. Reconcile lost acknowledgments, post-commit errors and cancellation. Same attempt/same
   inputs resolves to the established outcome; changed-input reuse is invalid. An
   already committed result is never reported as rollback.
7. Cleanup protects live publications/runs/readers using a qualified pin/maintenance
   protocol. Terminal unreferenced candidates and retired versions can be collected.

Reject conflicting stale-parent publishers and qualify concurrent first creation.
Blind append plus “choose newest” is insufficient. If public merge/update cannot
provide the required read-set/OCC semantics, qualify a Delta transaction extension;
never reinstate JSON CAS refs or add another durable coordinator. This is application
publication visibility, not a claim of native Delta multi-table transactions.

## Plan

### Milestones

| Milestone | Packages | Observable result |
|---|---|---|
| M1 — Native execution and target storage | UD00–UD03 | Native plans validate/write multiple Delta relations, publish coherently and reopen cold; old store protocol deleted |
| M2 — Source-derived process model | UD04–UD05 | Real heater/mixer sources in both state bases derive topology, properties, mathematics and support without predecessor graphs |
| M3 — Working simulator | UD06–UD08 | Case-bound initialized problems solve with native Ipopt; generated Pyomo and NL/SOL preserve the same problem semantics |
| M4 — Complete operating lifecycle | UD09–UD10 | Changes/reuse, maintenance, diagnostics, reconstruction and Rust/SQL/Python use the target exclusively |
| M5 — Qualified hard pivot | UD11–UD12 | Functional/extension/failure evidence passes, legacy paths are absent and G1–G7 close independently |

Execute in order in the same target implementation. Bring thin inspection/Python
consumers into earlier milestones where useful; UD10 completes their coverage.
Packages close only when behavior, necessary verification and deletion are complete.
Do not finish all reference-library breadth before the first source-to-solve example.

### UD00 — Target contracts and functional acceptance

**Depends on:** none. **Status:** Implemented in part / in progress.

The [simulator acceptance contract](../dev/simulator-acceptance.md) records exact
source declaration coverage, cases, independent oracles and tolerances. The terminal
recipe is implemented; its Rust/Python simulator targets are still pending and cannot
produce a successful receipt. ADR-0068 and blueprint revision 39 record the target.

- Record the target in a concise proposed ADR or coherent revision of proposed
  ADR-0067; supersede affected accepted decisions and explicitly amend the blueprint
  with its revision row. Cover D1/D3/D4/D6/D10/D11/D14 and affected §§3.3.3, 4, 5,
  14, 18–22, 24–25. Replace store/hash/commit constraints and the P10-only stop.
- Identify exact S01–S11/Slice A declaration coverage, source cases and independent
  engineering oracles/tolerances. Separate those from graph classes, byte encodings,
  pass numbers or row order inherited from the current implementation.
- Extend/replace existing xtask engineering acceptance with **Proposed recipe
  `just simulator-acceptance <output>`**. Reuse normal tests/fixtures; no new acceptance
  platform. Incomplete milestones must report incomplete, never success.
- Confirm the deletion scopes and useful reuse candidates, update generators and
  governance expectations, and refresh the environment as implementation starts.

**Surfaces:** ADR/blueprint/governance, schema declarations, source fixtures and xtask.
**Delete/replace:** stale restrictions, old object-preservation obligations and tests
whose sole purpose is certifying deleted implementation forms.
**Exit:** named functional oracles and a coherent target decision record exist before
related code changes. Records stay Proposed while acceptance gates remain open;
formal acceptance is not fabricated just to proceed with authorized implementation.

### UD01 — Exact libraries and shared native runtime

**Depends on:** UD00. **Status:** Implemented in part / in progress.

- Integrate exact Delta/kernel resolution and intentional features in Cargo/lock.
  Prove one family in the combined graph; use development APIs, not `DeltaOps` examples.
- Build shared RuntimeEnv and actual SessionState with composed Delta/PSE planners,
  native analyzer/optimizer/function/configuration/store/resource ownership.
- Replace root-only extension dispatch with normal planning and actual child plans.
  Keep opaque leaves only for real specialized algorithms/effects.
- Add executable tests for nested extensions, sentinel rule/UDF/settings survival,
  no internal default session, preparation/EXPLAIN effect timing and resource sharing.
- Test the write, type, publication, retry and codec seams listed above. Select supported
  routes and make a minimal pinned library patch only if the target requires one.
  Keep decisive probes as target integration tests instead of throwaway research.

**Surfaces:** Cargo/lock, catalog session/factory/planning/resources and required Delta integration.
**Delete/replace:** independent sessions, root dispatch, shadow registries and fallback paths.
**Exit:** V01/V02 integration passes; write/publication choices have runnable negative
and positive evidence. Displaying a node without executing its native children is insufficient.

### UD02 — Generated schemas and complete provider bindings

**Depends on:** UD01. **Status:** Implemented in part / in progress.

- Generate durable Delta schemas, Arrow projections, named conversions and invariant
  queries from one declaration. Explicitly retain identity, quantity, shape and status;
  restore extension metadata from those facts. Use lossless forms or explicit rejection.
- Bind workspace/publication/attempt scopes with native registries/providers. Exercise
  quoted names, aliases, faithful replacement, absence/errors, full/subset metadata,
  async resolution, table defaults, views, factories and parameterized table functions.
- Use native configuration/table properties first. Lower remaining domain policy and
  checks to common native admission/analysis/violation plans, covering no-scan plans
  and view inlining as well as ordinary scans.
- Use Delta scans, pruning/statistics and native datasource machinery. Advertise only
  established constraints and exactness; preserve residuals, multiplicity and limit semantics.

**Surfaces:** schema/registry/codegen, catalog providers/admission/policy and conversions.
**Delete/replace:** duplicate inventories, concrete admission rosters, editable metadata
copies, duplicate validators and wrappers that merely reimplement native behavior.
**Exit:** V04 boundary tests and V05 optimizer/provider checks pass. Every provider
level has an exercised contract; a new relation needs one semantic declaration. Generate
protected output through `just codegen`, never by editing generated files.

### UD03 — Delta writes, coherent publication and cold opening

**Depends on:** UD01–UD02. **Status:** Implemented in part / in progress.

- Implement one validating write route accepting a native input plan, actual session
  and commit properties. Bridge native insert/delete/update/merge/truncate, schema
  changes and maintenance to appropriate public Delta operations; no bypass route.
- Use native/Delta local checks and native candidate-wide key/FK/domain/completeness
  queries. Qualify adding CHECK constraints over existing rows and each public write path.
- Implement typed publication/member/attempt contracts with qualified OCC/read sets,
  first creation, stale-parent conflicts, uncertain outcome reconciliation and retry identity.
- Persist source payloads and durable contract/model/case/result/provenance facts in
  Delta. Truly opaque artifacts use typed relations and provider/sink boundaries.
- Open exact-version providers from a fresh process; avoid unconditional full-table
  loading just to reconstruct a catalog. Required semantic checks remain explicit.
- Replace all old durable-store callers in this cut and recreate fixtures from source.
  Partial attempts are inspectable but never masquerade as complete published models.

**Surfaces:** catalog store/snapshot/inspection, compiler commit/memo, source/export
callers, lifecycle tests and fixture tooling.
**Delete/replace:** JSON refs/manifests, custom CAS/local control, alternative durable
IPC/Parquet membership, encoding policies, sidecars/stage/context indexes and the old
publication-journal objects. Preserve required outcome meaning in new relations.
**Exit:** M1 and V03/V06/V07 pass, including failures after every candidate/root step
and cold reopen. No product caller can reach the old store.

### UD04 — Source authoring, editing and native normalization

**Depends on:** UD03. **Status:** Implemented in part / in progress.

- Connect package resolution, typed parsing, source spans, changes, renames and primitive
  validation to target providers/commands. The parser is a contracted input transform.
- Use native plans for correspondence, defaults, joins, selectors, domains and normalized
  outputs; remove repeated decode/collect/rebind work without a functional boundary.
- Persist useful checkpoints, not every former pass output or receipt. Do not invent a
  Delta transaction per internal transform to preserve the old stage choreography.
- Produce heater/mixer sources in FTPx/FcTP, publish normalized facts and reopen/query
  them. Exercise bad sources/quantities, dangling targets, identity-preserving rename,
  missing/default values and conflicting changes using independent assertions.

**Surfaces:** authoring, source/commit/normalization, native functions and early acceptance.
**Delete/replace:** old document/sidecar restore, change persistence, imperative relational
normalization and redundant AST mirrors/source orchestrators.
**Exit:** S01 and a real source→publication→cold-query path pass. No prebuilt graph or
old store may supply the model fixture.

### UD05 — Native inference, provenance and symbolic construction

**Depends on:** UD04. **Status:** Implemented in part / in progress.

- Implement features, domains, containment, topology/ports, property demand, method
  choice, contributions and laws with native plans/functions, joins, grouping/windows,
  nested operations and set semantics.
- Prefer native recursion where truth/multiplicity/support match. Otherwise use one
  native fixed-point operator with actual input plans, explicit strata, cancellation,
  finite termination and incomplete failure; remove the separate rule execution pathway.
- Carry support and absence domains as data. Share pure relationship functions where
  arguments contain all facts; whole-relation checks/provenance remain native queries.
  Do not infer lost domain lineage from optimized text or scan names.
- Build compact typed symbolic facts in the same framework. Use native Expr where
  faithful, and relations/extensions for remaining indexed/implicit/derivative/guard
  meaning. Register domain functions with complete typing/field/coercion/nullability/
  volatility and valid simplification/property hooks, shared by applicable consumers.

**Surfaces:** rules, compiler P4–P10 responsibilities, math/quantity/material/templates,
kernel declarations and native functions.
**Delete/replace:** procedural relational assembly, closed rule/truth/type interpreters,
post-hoc lineage recovery and predecessor graphs. Derived graph layouts require a
real algorithm consumer; old graph types have no automatic retention entitlement.
**Exit:** M2, S02/S03 and V05/V09 pass for all four source unit/state combinations,
including independently checked term multiplicity, quantity meaning and support removal.

### UD06 — Case-bound problems, structure, scaling and initialization

**Depends on:** UD05. **Status:** Proposed / open.

- Implement the functions of P11–P15: appropriate discretization, index expansion,
  case overlays/substitution, bounds, objectives and variable/equation ordering.
  Preserve free indices until expansion is needed; do not preserve old object choreography.
- Qualify a small analytic derivative/integral discretization with boundary conditions,
  units, stencil identity and missing-policy rejection. This does not certify all
  dynamic or distributed-physics applications.
- Derive incidence with native relational work; use native physical extensions for
  genuinely specialized matching/SCC/block algorithms and explicit identity mappings.
- Construct required nominal scaling, block-triangularization and single-control-volume
  initialization as native operation plans over declared model/case/static facts.
  Execute their data-only steps here and their solver steps in UD07.
- Distinguish unknowns from absent guesses, fixed/free treatment, invalid bounds,
  under/overdetermination, missing derivatives and unsupported operations before consumers.

**Surfaces:** compiler lowering, structural/plans/math contracts, case/runtime relations
and native extensions.
**Delete/replace:** independent orchestration, mutable model/case stores, parallel graph
or incidence authority and placeholder implementations with no target behavior.
**Exit:** S04 and structural V09 pass. A source-derived case yields a complete supported
problem and initialization plan; malformed cases have independently expected diagnostics.

### UD07 — Evaluation, derivatives and native Ipopt simulation

**Depends on:** UD06. **Status:** Implemented in part / in progress. Independent
native expression preparation and actual Ipopt execution are implemented with focused
tests. Product planner installation, source-bound problems, complete kernel coverage,
durable run publication and the UD06 dependency remain open. The numerical
mixer/heater test constructs expressions directly and does not close Slice A.

- Lower supported mathematics to a reusable numerical evaluation program or native
  expression/kernel combination. Use built-ins/vectorized operations where faithful;
  preserve guards and numerical policy. Never rebuild a query/model per solver callback.
- Implement residual/Jacobian evaluation, sparsity and kernel derivatives. Hessian
  capability/policy is explicit; limited-memory is a selected profile, not silent fallback.
- Implement actual Ipopt FFI in the native execution operator. It owns per-run mutable
  workspace, callbacks, cancellation, thread limits and results. Account for numerical/
  foreign allocations as well as memory tracked directly by DataFusion.
- Execute initialization/solve, then publish input bindings/settings, values, residuals,
  status and diagnostics through the common Delta route. Solve completion and durable
  result publication have separate, truthful outcomes.
- Qualify a connected feed→mixer→heater→product flowsheet, nonlinear property case and
  infeasible/failed/cancelled runs with independent physical and derivative oracles.

**Surfaces:** numerics, native backend/Ipopt boundary, kernels, native operators and results.
**Delete/replace:** solver stubs, direct product execution outside the native framework,
separate run scheduler and mutable result store. Reuse useful FFI/algorithms in target.
**Exit:** S05/S11 and native M3 pass with a real solver. Analytic/finite-difference
Jacobian checks and physical residual tolerances hold; a mock or plan display cannot pass.

### UD08 — Generated Pyomo and NL/SOL routes

**Depends on:** UD06–UD07. **Status:** Proposed / open.

- Generate Pyomo bundle/adapter and NL writer/SOL reader from the same problem/kernel
  contracts. Use coarse Arrow boundaries; backend layout/code is derived, not another model.
- Expose backend preparation/invocation/export/result ingestion as native operations
  with explicit IO/effect/resource contracts. Export alone is distinct from execution.
- Bind kernels, derivatives, scales, source maps, bounds and options. Report actual
  unsupported capability; never silently switch backends or reinterpret mathematics.
- Qualify generated routes against independent engineering assertions and native
  results under the declared tolerance policy. Shared lowering means backend agreement
  alone cannot prove scientific correctness.

**Surfaces:** Pyomo/NL backends, generated Python adapter, kernel generation and native nodes.
**Delete/replace:** handwritten parallel backend semantics, direct Python model-mutation
routes, legacy export envelopes and placeholder APIs.
**Exit:** S06 and remaining M3 cases pass with actual construction/execution/result
consumption for supported Slice A problems, including NL/SOL interoperability.

### UD09 — Dependencies, reuse, CDF and target retention

**Depends on:** UD03, UD05–UD07. **Status:** Proposed / open.

- Record actual input versions/slices, absence scopes, contracts/policies, function/
  kernel/backend versions and external observations. Implement qualified exact reuse;
  recompute conservatively when finer reuse is unproved.
- Feed CDF into native semantic-diff/dependency queries with explicit version ranges
  and preimage/postimage meaning. Qualify a real edit's impact/reuse; CDF alone is not
  an incremental compiler.
- Use Delta compaction/checkpoint/pruning/maintenance. Derive retention from live
  publications, retained runs and readers; coordinate pin acquisition and cleanup.
- Qualify data-file and log/checkpoint retention separately. Use `with_keep_versions`
  for its actual scope. Refuse destructive maintenance where the supported backend
  cannot establish the required reader-exclusion/pinning guarantee.
- Collect terminal unreferenced candidates and retired target versions; do not rebuild
  stage sidecars, artifact-hash stores or indefinite history preservation.

**Surfaces:** dependency/reuse queries, Delta control/attempt/run relations and maintenance.
**Delete/replace:** artifact-hash memo protocol, context/stage restoration, duplicate
invalidation tracking and bespoke Parquet maintenance.
**Exit:** S08/V10 pass for data/policy/absence/function changes, CDF update pairs,
compaction and cleanup/reader races. Clean target recomputation is the reuse oracle;
this is not old-implementation equivalence.

### UD10 — Inspection, Python ownership and supported reconstruction

**Depends on:** UD02–UD09. **Status:** Proposed / open.

- Expose model/case/problem/run/support/diagnostics/settings through the same Rust,
  SQL and Python hierarchy, native metadata and domain views over actual bindings.
- Finish async resolution, snapshot/observation distinction, metadata coverage,
  external factories and table functions under common policy. Unity/remote services
  remain eligible extensions, not prerequisites for local simulator functionality.
- Preserve Arrow stream/schema ownership, cancellation and release across Python/FFI;
  use streams and deliberate materialization without a second Python query session.
- Support qualified logical/provider descriptors with exact snapshot/contract/runtime
  rebinding and fresh physical planning. Unknown versions, missing owners or unsupported
  codec arms fail clearly; no panic or hidden effects.
- EXPLAIN, discovery and serialization do not write tables or solve models. Metrics
  and provenance describe actual execution; domain failures remain structured.

**Surfaces:** inspection/resolution/codecs, pse-py, Python contracts/interfaces and consumers.
**Delete/replace:** old store/manifest handles, independent inspector logic and duplicate
settings/provider/stream representations.
**Exit:** S09/M4 and V08/V11 pass. Fresh Rust/Python processes see the same published
solved model/run without predecessor objects or digest-based authority reconstruction.

### UD11 — Functional qualification, extension proof and cost measurement

**Depends on:** UD00–UD10. **Status:** Proposed / open.

- Complete the terminal simulator recipe using fresh sources, normal tests and target
  stores: model/case construction, actual solve, publication, cold Rust/Python reads,
  inspection and meaningful edit/recompute/reuse.
- Qualify the Slice A declaration families and ordinary extensions: new property
  method, invariant and provider/parameterized relation. One declaration plus necessary
  implementation/tests must suffice and every native policy boundary must apply.
- Attack false constraints/pushdown, no-scan/view bypasses, duplicate/null/unknown data,
  invalid quantities, cyclic/ambiguous models and unsupported kernels; include the
  review's lifecycle, concurrency, cancellation and resource cases.
- Measure cold/warm construction, planning, checks, Delta IO/log replay, solve, export,
  inspection, peak/retained memory and file/log growth for small and scaled indexed models.
  Fix correctness first, then actual bottlenecks with native mechanisms where suitable.

**Surfaces:** ordinary tests/fixtures, existing acceptance and benchmark tools/evidence.
**Delete/replace:** P10-only terminal success, old golden stores, format-preservation
assertions and abandoned harnesses with no target consumer.
**Exit:** S01–S11/V01–V13 have fresh named receipts. No successful receipt with a stub
solver, skipped required case, incomplete outcome or stale editable extension.

### UD12 — Final deletion and independent gate closure

**Depends on:** UD11. **Status:** Proposed / open.

- Audit reachable product execution/storage paths and the ledger below. Delete remaining
  legacy modules/types/formats, callers, fixtures, dependencies and stale governance.
- Use skill capability-gap queries as syntax leads and inspect actual consumers;
  a search count alone cannot establish absence of bypasses.
- Run appropriate final Rust/Python/generation/governance/feature/docs checks, baseline
  0; fix findings. Earlier receipts do not certify later source changes.
- Review the implemented target against G1–G7 independently; update records/status
  truthfully. Formal ADR acceptance follows the normal decision process and required review.
- Record delivered scope, engineering evidence, costs, future extensions and outcome
  sections. No unresolved required simulator/lifecycle gate can be called completion.

**Exit:** every package and S01–S11 is complete, all seven gates pass for the supported
scope, required checks are green and no legacy/compatibility runtime path remains.

### Replacement and deletion ledger

Scopes identify where to work, not boundaries to preserve. Renaming a mechanism does
not delete it; retaining it solely in fixtures is not a successful cut.

| Existing surface | Target disposition | Deadline |
|---|---|---|
| refs/manifests/control/local CAS, encoding/membership, sidecar/stage/context stores | Delta tables/native operations plus typed publication control | UD03 |
| Snapshot/ManifestRef/revision receipts/publication journal objects | Target publication/member/run/outcome relations and actual bindings; replace all old callers in this cut | UD03 |
| Private mutable-copy factories and separate mutation/validation routes | One validating Delta route; native temporary worktables only for actual temporary work | UD03 |
| Root OperationNode dispatch and relational callback envelopes | Native composed planner/children; contracted specialized effect/algorithm nodes | UD01; consumers through UD08 |
| Duplicate catalog/role/source/policy/settings inventories | Native hierarchy/configuration plus one domain binding/policy definition | UD02 |
| Repeated SessionContext creation, collect/decode/rebind | Shared actual runtime/session, streams and explicit algorithm boundaries | UD01, UD04–UD07 |
| Procedural join/group/selection/normalization and closed rule interpreter | Native plans/functions, recursion or contracted fixed-point node | UD04–UD05 |
| Predecessor graph stores / parallel model ASTs | Typed target mathematics; derived layouts only where consumed by a target algorithm | UD05–UD06 |
| Numerical/backend stubs and direct solver/Python product paths | Actual native physical operators and generated bindings | UD07–UD08 |
| Artifact-hash memo / restore sidecars / change trackers | Typed dependencies, native queries/CDF and qualified reuse | UD09 |
| Independent inspectors/codecs/old Python handles | Same providers, owned streams and qualified reconstruction | UD10 |
| Historical runtime stores/goldens/compatibility fixtures/generators | Fresh target sources, independent assertions and target generation | Every cut; UD12 audit |

Historical Markdown receipts can remain clearly marked historical; they are not data
objects used by the codebase. No dormant runtime schemas, fallback imports or test-only
old reconstruction engine are retained to interpret them.

### Review and inherited functional traceability

| Review finding | Packages | Review checks |
|---|---|---|
| F01 — model publication | UD01, UD03, UD09 | V06–V07, V10 |
| F02 — representation | UD02, UD05–UD08, UD10 | V04, V09, V11 |
| F03 — write validation | UD01–UD03 | V03 |
| F04 — commit/retry outcomes | UD01, UD03, UD07, UD09 | V07–V08 |
| F05 — native composition | UD01, UD04–UD08 | V02, V09 |
| F06 — common policy | UD01–UD03, UD10 | V02, V08 |
| F07 — bespoke artifact lifecycle | UD03, UD09–UD12 | V01, V06, V12 |
| F08 — math/numerical meaning | UD05–UD08, UD11 | V09, V13 |
| F09 — lineage/dependencies | UD04–UD05, UD09–UD10 | V05, V10 |
| F10 — codecs | UD01, UD10 | V11 |
| F11 — resources | UD01, UD03, UD07–UD08, UD10–UD11 | V08, V13 |
| F12 — CDF/retention | UD09 | V10 |
| F13 — dependencies | UD00–UD01 | V01 |
| F14 — policy/decision replacement | UD00, UD12 | V12 |

Plan 06's useful functions carry by outcome: hierarchy/admission/settings/resources →
UD01–UD02/UD10; commands/publication → UD03; authoring/normalization → UD04;
inference/quantities/math/provenance → UD05–UD06; cold inspection/reuse → UD03/UD09–UD10;
engineering/deletion → UD11–UD12. Its old objects, packet dependencies and P10-only
stopping point do not carry forward.

## Verification

### Evidence and command schedule

**Proposed:** target runtime acceptance is unrun. Interface-checked review evidence
narrows investigation but does not close behavioral tests. Receipts record exact
commands, source/lock identity, modes/features, input conditions, counts and baseline
0. Characterizing an upstream gap is distinct from qualifying the selected target route.

Use existing recipes. New names below are **Proposed**, created in their packages;
never report them as existing/passed before they are implemented.

| Purpose | Command / proposed target | When |
|---|---|---|
| Environment/dependency graph | `just doctor`, `just metadata`, `just family-check`; locked full metadata when dependencies/features are needed | UD01 and dependency changes |
| Native/Delta/provider integration | `just test-package pse-catalog --test unified_delta_contracts` | UD01–UD03, UD09–UD10; create target there |
| Publication/recovery/retention | `just test-package pse-tests-lifecycle --test delta_publication` | UD03/UD09; create target there |
| Source/semantic/simulator behavior | `just test-package pse-tests-engine --test unified_simulator`, focused domain/kernel tests | UD04–UD11; create target there |
| Numerics/backends | Focused backend/conformance targets; `just parity-container` for exercised IDAES 2.12.0 behavior | UD07–UD08/UD11; missing required solver fails |
| Rust final gate | `just ci-fast` or appropriate constituent recipes during cuts | Focused during work; complete final source UD12 |
| Generation/invariants | `just codegen-check`, `just governance`, `just family-check`, `just adr-lint`, `just lint-agents` | Affected cuts and UD12 |
| Actual Python boundary | `just py-sync`, `just py-test`, `just quality` | UD10 and final current-source qualification |
| Terminal simulator/cold-open/cost | Proposed `just simulator-acceptance <output>` replacing/extending existing xtask engineering workflow | Early incomplete milestones; complete UD11/UD12 run |
| Documentation | `just docs`, focused links/spelling/plan-coverage checks | Documentation cuts and final source |

Rust test recipes retain explicit force-validation. Run necessary focused checks
following each cut; broad checks when the connected target is ready. Do not repeatedly
run full suites after metadata-only changes. Distribution qualification remains
release-time work, outside this local pivot's critical path.

### Terminal functional journeys

Fresh sources and a new target Delta store are required. Missing prerequisites fail;
no required case is skipped. These journeys supplement the declaration-family
coverage set from UD00, rather than substituting for it.

| Journey | Required oracle | Outcomes / review checks |
|---|---|---|
| Heater/mixer × FTPx/FcTP | Independent domains/topology/property/equation/quantity/balance/support assertions | S01–S04; V04–V05, V09 |
| Connected flowsheet and nonlinear property solve | Actual native Ipopt; independent material/energy balances, bounds, status and physical residual tolerances | S02–S05/S11; V09, V13 |
| Pyomo and NL/SOL | Actual supported construction/execution/result ingestion; same problem meaning plus independent physical oracle | S06; V09, V13 |
| Indexed/guard/derivative seam | Compact meaning before expansion, valid tuples/ordering, analytic derivative/stencil expectations and inactive-branch safety | S03–S05; V05, V09 |
| Publication/cold Rust and Python | Exact versions and complete model/case/run; equal typed queries and owned streams from fresh processes | S07/S09; V04, V06, V08, V11 |
| Meaningful edit and reuse | Changed data/topology/policy/absence invalidates; clean target recomputation agrees; unaffected reuse is justified | S01/S08; V05, V10 |
| Invalid/unsupported input | Quantities/references/bounds/keys/FKs, tags, ambiguous methods, incomplete demand/closure, missing derivatives/backend capability reject at named boundaries | S02–S06/S11; V03–V05, V09, V11 |
| Adversarial lifecycle | Competing publishers, lost acknowledgments/hook errors, cancellation/resources, partial streams and cleanup/pin races preserve declared outcomes | S07–S09/S11; V06–V08, V10 |
| Ordinary extension | Method, invariant and provider share discovery/policy/execution/provenance/persistence rules | S10; V01–V03, V08, V12 |
| Cost/resource envelope | Small/scaled indexed models; recorded cold/warm phase costs, peak/retained memory and file/log growth | S11; V08, V13 |

Record per-oracle tolerances and scientific assumptions. Do not impose old object
bytes or blanket bitwise equality. Backend agreement alone is insufficient when it
shares the same lowering error. Measure actual costs; no inferred library speedup.

### Independent gate closure

| Gate | Current state | Closure evidence |
|---|---|---|
| G1 — Authority | Fail for the target cut | Old compiler/store/Python authority remains beside Delta publication; remove old writer/read authority in UD03 and audit absence in UD12; completion review §6 |
| G2 — Fidelity | Unresolved | Required type/value/model/math/backend round trips and engineering outcomes; UD02/UD05–UD08/UD10 |
| G3 — Validity | Unresolved | All write/execute routes enforce local/cross-table/domain obligations; UD02–UD07 |
| G4 — Hidden behavior | Unresolved | Actual planner/session and effect timing across SQL/Rust/Python, views, functions/factories/codecs; UD01/UD03/UD10 |
| G5 — Consistency/recovery | Unresolved | Publication, conflict/retry/cancellation reconciliation and protected readers; UD03/UD07/UD09 |
| G6 — Transformations/reuse | Unresolved | Truthful properties, multiplicity/lineage/numerical policy, complete dependencies and CDF/reuse; UD02/UD05–UD09 |
| G7 — Capability claims | Unresolved | Combined build and actual simulator/backend/inspection journeys, complete extensions, explicit unsupported behavior; UD01/UD07–UD12 |

No aggregate score offsets an unresolved gate. Registration, static checks, matching
crate versions and EXPLAIN output do not establish the whole target.

### Planning-document evidence

**Interface-checked:** based on the completed review and exact-pin evidence, live
implementation boundary, blueprint functional inventory, skills and recipe surface.
No runtime implementation or target acceptance is claimed.

**Tested, documentation/static scope, baseline 0:**

- `just docs`: **0 build failures, 1 large-search-index warning**. This builds the book,
  not the simulator. The warning remains reported rather than treated as a clean gate.
- `just lint-agents`: **0 findings** across 22 files, 75 path references, 57 recipe
  references and 7 allowlist entries.
- `.venv/bin/typos AGENTS.md README.md STATUS.md docs/plans/README.md docs/plans/06-provider-contracts-hard-pivot.md docs/plans/07-unified-datafusion-delta-hard-pivot.md docs/SUMMARY.md`:
  **0 findings**. The repository spelling recipe has no file-selection parameter.
- Targeted front-matter, local-link, ADR-reference and coverage checks: **0 errors**;
  13 ordered packages, 14 mapped findings, 11 outcomes, 13 review checks and 7 explicitly
  unresolved gates. Scoped `git diff --check` also reports **0 whitespace errors**.

At plan creation, `just doctor` had the environment failure recorded above, and
runtime/native/Python checks had not run. The later implementation receipts below
supersede that planning-time environment and execution status.

## Open items

These are owned implementation decisions with defaults and finite checks, not
requests for another planning phase or permission to preserve legacy mechanisms.

| Item | Default / resolution rule | Owner / deadline |
|---|---|---|
| Exact publication operation | Typed Delta control table with expected-parent read-set/OCC behavior; prove first-create/update/retry, adjust native integration if necessary | Codex implementation stream; UD01 probe, UD03 closure |
| Public write-plan visibility | Validating builders and explicit input/effects; smallest pinned prepare/execute API extension only if needed | Codex; UD01–UD03 |
| Durable encodings | Generated lossless representation or explicit rejection; no opaque universal model | Codex; UD02 |
| Symbolic/native expression boundary | Native semantics first, otherwise typed symbolic relations/extensions; derived layouts require actual consumers | Codex; UD05–UD07 |
| Exact Slice A declarations/tolerances | Blueprint functional inventory plus independent engineering equations; explicit coverage before certification | Codex; UD00 definitions, UD11 proof |
| Target retention | Protect live publications/runs/readers; qualify file/log retention and pin/cleanup races | Codex; UD09 |
| Remote-service breadth | Keep native extension seams; local completeness does not require Unity/distributed execution/every cloud store | Future functional extension unless a required case needs it |

Module boundaries, package granularity and API spellings may change with evidence.
Required functional outcomes and coherent-publication/type semantics may not silently
shrink. Any claimed fundamental obstruction needs a concrete attempted native route
and evidence, resolved against the two controlling objectives.

## Outcome (recorded after implementation)

### What was built

UD00–UD05 foundations and UD07 native numerical preparation/Ipopt execution are
implemented in part, as recorded below. No package or
milestone is closed: source-derived simulator execution and the legacy deletion cuts
are still outstanding. Inherited functionality does not close these packages.

### A mistake made and corrected

Exact-pin assumptions that failed executable probes were corrected:

- `SaveMode::ErrorIfExists` alone did not prevent two initial control writers from
  rebasing into successive commits. Publication control now disables Delta commit
  retries, uses conditional parent updates, and reconciles retries explicitly.
- Setting logical projection metadata did not survive every native rewrite. The
  Delta physical input uses the native `ProjectionExec` schema-metadata override;
  execution annotations no longer leak into the durable schema. The same exact-field
  boundary restores generated annotations on typed publication streams; the real
  source-reparse test caught their absence on ordinary SQL output.
- Delta's default MERGE pre-scan consumed a native physical source before its actual
  merge. Native streaming mode disables that pre-scan; binary-identity self-merges
  now reach the same declared CHECK as other edits. Native optimizer display aliases
  are removed from value predicates before Delta serializes them.
- `WriteBuilder` cannot create CHECK configuration directly at this pin. Declared
  input is guarded by a native assertion, then Delta's constraint builder validates
  and persists the CHECK before the write operation reports completion. A later
  failure retains the already committed unpublished version in its error.

### Deviations from the plan, deliberate

None recorded. Changes to architecture decisions require corresponding records;
routine target-compatible implementation choices require no new approval cycle.


## Implementation receipts — 2026-09-15

### Implemented in part

- ADR-0068/blueprint revision 39 and the Slice A acceptance contract/recipe.
- Exact Delta `58f07cd62bfbce3649a7e1c87c696288068ae184` and resolved kernel
  `8ba063f8f84fec222000f66d40d70911d7c79675`; shared native planner composition.
- `pse_catalog::delta::write::DeltaWrite`: native logical and physical child,
  validating public builder, actual session, commit properties and once-only execution.
- Registry-derived durable layouts, native encode/decode UDFs, full unsigned range,
  nested/null-parent/identity/nanosecond restoration and native exact-version views.
- Generated `runtime.publications` control relation. `DeltaPublish` consumes a real
  control-row input, verifies selected members and inputs, checks an expected parent,
  and commits the entire head/member vector in one Delta transaction.
- Commit metadata indexes attempts; reconciliation checks the complete typed row at
  the actual matching Delta version. Same-request retries work after the head moves;
  changed-request identity reuse fails. Missing history cannot authorize a retry.
- First creation and updates disable automatic commit rebasing. Actual object-store
  faults after successful control writes are reconciled without duplicate commits.
- Native grouping/anti-join plans establish keys and declared references. The reusable
  `pse_assert_relation` UDF applies the existing domain value predicates. These checks
  do not yet establish all process-model completeness or nested ordinal obligations.
- Cold publication opening binds native catalogs/schemas/views. Each invocation gets
  private native registries sharing exact providers and caller resources. Delta table
  opening uses the caller's native object-store registry.
- Existing common effect admission recognizes both Delta commands, including nested
  commands without scans. The old operation framework still awaits replacement.
- Publication members and inputs pin the semantic contract fingerprint as well as
  relation ID/version; altered declarations cannot silently reinterpret stored values.
- `WritableTable` bridges native INSERT/overwrite/UPDATE/DELETE/TRUNCATE/MERGE hooks
  to Delta's validating builders. Insert/merge retain real physical children, every
  invocation preserves the caller session, and prepared commands execute once.
  Scans retain native Delta pushdown/statistics and exact snapshot behavior. REPLACE
  requires explicit MERGE keys/clauses rather than inventing a conflict key.
- Registry-derived `DeclaredCheck` persists a fingerprint-named UDF in a native Delta
  CHECK. Declared SQL edits enforce local enum/identity/nested value rules with the
  same validator used by publication admission. The actual function is rebound from
  the registry; arbitrary stored function names do not supply implementations.
- Native delete metrics provide affected counts; when imported files lack row
  statistics, native aggregation counts the exact before/after snapshots without
  re-evaluating the delete predicate. Mutation errors preserve underlying typed causes
  and distinguish an observed commit followed by failed work from an unresolved outcome.
- `publication_plan::plan` composes declared member writes, native struct expressions,
  deterministic member aggregation and conditional publication. Actual write versions
  supply the record; callers do not collect versions and reconstruct control rows.
  Focused composition qualification passes; it does not close M1. Replay of the whole
  write graph still needs request/dependency recovery; the existing complete-record
  publication retry proof does not establish automatic replay of member writes.
- A plain `SessionFactory::from_builder` receives the composed planner; explicitly
  supplied planners remain intact. Factories can open target publications with their
  retained native configuration, function/rule implementations and runtime resources.

- Authored and normalized document contracts now carry exact `source_text`. The
  loader emits the original UTF-8 text, and compiler source consumers reparse those
  typed rows. Deleted the separate document blob path, prepared put/read commands,
  `DocumentArtifact`, and duplicate source-version/change inventories in custom
  receipts. Remaining custom store and compiler execution machinery is still open.
- Native source-span plans derive nested projections, list unnesting, byte-length
  comparisons and joins from the declared field annotations. They reject missing
  documents and out-of-range spans against the exact selected document relation.
- `Publication::relation_stream` exposes one selected relation with its generated
  Arrow schema. A native projection restores schema annotations after checking exact
  fields; an arbitrary SQL result does not automatically claim a relation contract.
  The stream retains native provider/resource owners after its publication is dropped.

- `pse_authoring::native::relation_plan` groups exact document rows by package,
  invokes the existing bounded parser as a typed native UDF, then unnests/projects
  declared source facts. Header/package identity, document/path identity, duplicate
  paths, syntax, parse limits, cancellation and allocation refusal keep typed causes.
  Preparation and physical planning do not parse. It is consumed inside the composed
  Delta publication graph; no output-fact batch is supplied by a predecessor stage.
  Separate relation consumers currently parse independently; shared execution and
  scaled-workload cost qualification remain open.

- Provider capture now executes its actual native scan, local validation UDF and
  partitioned window key assertions. The callback that replanned/rescanned captured
  inputs is deleted. Runtime preparation accepts directly bound native function
  owners; SQL registration is for name resolution. The diagnostic codec still
  refuses functions it cannot reconstruct faithfully.
- Constructed inference facts now own checked values, materialized support and typed
  derivations, without predecessor sessions, witness objects, ancestor objects or
  completed-plan collections. Completed program/member handles likewise release
  producing sessions/programs. The explicit algorithm adapters retain their decoded
  arguments and allocation claims, without redundant source/execution keep-alives.
- Intermediate materializations and captured facts share a read-only provider over
  leased Arrow buffers, using native `MemorySourceConfig` scans. `MemTable` was
  inspected and rejected for this boundary because it exposes mutable partitions
  and mutation hooks. The separate `ComputedTable` and its retained producer graph
  are deleted. Source correspondence remains checked before support materialization.
  Captured providers also retain only completed buffers and registry declarations;
  their completed-computation accessor/producer owner is deleted. Lifetime assertions
  keep captured facts alive while proving the producing provider is released.
  The remaining old store, compiler choreography and fixed-point driver remain open.
- P4 now declares the physical quantity and kernel relations its predicate evaluator
  actually reads. These dependencies are generated from the existing declaration;
  `just codegen` regenerated all three schema targets after the integration failure.
- UD07's first numerical consumer compiles native residual/Jacobian expressions once
  against the actual caller state, with sparse derivative coordinates and finite-value
  assertions. It differentiates actual native implementations rather than SQL names,
  preserves native CASE execution and refuses missing derivative bindings. Scalar
  mathematical facts lower into this same program; graph layouts are borrowed during
  preparation and are not retained for evaluation. Native projections retain their
  function/allocation owners after the program handle is dropped. Arbitrary immutable
  fixed-parameter transformations require no derivative binding or numeric argument
  restriction. All 14 supported unary derivative bindings and variable-power derivatives
  have independent analytic/finite-difference oracles. The scalar adapter retains
  ordered affine/weighted arithmetic and explicit unit conversion, and refuses
  unexpanded indices, missing symbols and inexact integer conversion.
  No Hessian, kernel derivative, complete case/problem, initialization, Ipopt, Pyomo
  or NL/SOL result is claimed. Full resource/cost qualification remains open; conservative
  expression reservations and lifetime tests are bounded implementation evidence.

### Tested

All Rust test receipts use `pse-relations/force-validate`, baseline 0, and nextest
default unless another profile is named.

| Command | Result and scope |
|---|---|
| `just test-package pse-catalog --test provider_contracts --test captured_native_layouts --no-fail-fast` | **23 passed, 0 failed, 0 skipped**, run `9132a047-7122-4df2-956c-03873072750f`. Native capture, partition-wide key checks, SQL null semantics, literal dotted names, bound-function ownership, producer release and read-only materialized buffers. |
| `just test-package pse-tests-engine --test native_template_graph --profile ci --no-fail-fast` | **1 passed, 0 failed, 0 skipped**, CI/force-validation/baseline 0, run `02886b50-7abf-4aab-8926-aa1fab436fc4`, 236.131 seconds. Actual source-to-P10 symbol/expression/equation/quantity assertions. The default-profile rerun timed out at 120 seconds; its zero-failure gate remains open. This uses predecessor compiler/store contracts and does not qualify the target Delta path. |
| `just test-package pse-numerics --no-fail-fast` | **11 passed, 0 failed, 0 skipped**, run `21072b96-81d6-435e-97bd-1ec07bca62f7`, default/force-validation/baseline 0. Actual native projection and repeated prepared evaluation, sparse Jacobian order, nonlinear analytic/finite-difference checks, CASE/domain/overflow behavior, actual UDF binding, unrestricted fixed parameter expressions, resource/cancellation distinctions, output reservation lifetime and scalar mathematical lowering after graph release. |
| `cargo clippy --locked -p pse-numerics -p pse-catalog --lib --tests --features pse-relations/force-validate -- -D warnings` | **0 project Clippy findings**, dev/locked/force-validation/baseline 0, after the numerical and immutable-materialization cuts. Transitive future-incompatibility warning remains. |
| `just governance` | **59 passed, 0 failed, 0 skipped**, run `cd441aab-7995-44dc-a05d-42a1343ac0a6`, default/force-validation/baseline 0; family check also passes. This precedes the final scalar-lowering module and existing-crate test dependency. |
| `just test-package pse-rules -p pse-compiler --no-fail-fast` | **99 passed, 0 failed, 0 skipped**, run `83d27baf-5d48-4457-89c0-9e48ce9637d4`, 10.265 seconds. Includes source-snapshot/union-parent/completed-program release assertions, exact row/absence/branch support, finite recursion, conflict/quantity/normalization checks and affected algorithm consumers. This is not simulator or target Delta inference qualification. |
| `just test-package pse-catalog --test unified_delta_contracts --test unified_delta_dml --no-fail-fast` | **21 passed, 0 failed, 0 skipped**, run `d466e588-abc1-446c-a0d5-6f2b3919d40e`. Native child/session preservation, declaration CHECK constraints through SQL, lossless layouts, cold views, generated controls, two-writer creation/update, retry/identity checks, actual lost acknowledgments, candidate key/reference/value/fingerprint failures, input availability, no-scan effect policy, real-file counts without statistics, and composed two-member publication with quoted names. |
| `just test-package pse-tests-engine --test unified_sources --test catalog_store_protocol --no-fail-fast` | **14 passed, 0 failed, 0 skipped**, run `46c2c490-7c79-429c-b529-c7b54502d63b`. Real YAML/TOML package → native Delta publication → fresh native context → owned typed stream → exact UTF-8 reparse; source rows and species agree. Includes 13 affected consumer regression tests. This does not qualify heater/mixer normalization or a separate OS process. |
| `just test-package pse-catalog --lib delta::source_spans --no-fail-fast` | **1 passed, 0 failed**, 111 unrelated tests filtered, run `b2b2e942-f2f1-4818-89b4-cfcb0061f290`. Native nested-list/struct source checks, multibyte UTF-8 byte extents, missing documents and null/empty containers; reversed spans fail existing local admission. |
| `just test-package pse-tests-engine --test unified_sources --no-fail-fast` | **1 passed, 0 failed, 0 skipped**, run `be485ce8-676d-49f1-aca2-7b2f6c403689`, 2026-09-16 continuation. Native grouped parser feeds declared Delta writes directly from document rows, publishes, opens in a fresh native context and reproduces exact original text and generated facts. |
| `just test-package pse-authoring --test native_parser --no-fail-fast` | **2 passed, 0 failed, 0 skipped**, run `b9d28fc2-4a13-4524-9f8f-0ae8e550434b`. Deferred parsing, typed invalid-source causes, identity/grouping, duplicate paths, parse limits, cancellation, allocation failure and released reservations. |
| `just test-package pse-authoring --no-fail-fast` | **42 passed, 0 failed, 0 skipped**, run `a80cd30e-d72e-4071-ad31-f42d5f3ae026`; affected loader/editor/rename/ownership regressions, before the two new parser tests. |
| `just test-package pse-tests-engine --test commit_p0_p2 --test source_projection_admission --test native_normalization --no-fail-fast` | **11 passed, 0 failed, 0 skipped**, run `481f7974-cb8a-434e-afa3-91a3222e7fec`; affected source consumers, 652.908 seconds. This does not certify their legacy storage/execution architecture as the target. |
| `just test-package pse-catalog --no-fail-fast` | **188 passed, 0 failed, 0 skipped**, run `5a71706d-791c-4f93-a3c6-d52a73830605`; includes the catalog targets before the source-text cut. This is a regression receipt, not an old/new architecture equivalence campaign. |
| `just test-package pse-tests-governance -p pse-relations --test delta_revisions` | **1 passed, 0 failed, 0 skipped**, run `9c7b0b94-40a7-476d-aed2-3527a5fa0c65`; full source revisions in manifest/lock. |
| `just family-check` | **0 findings**; Arrow/Parquet 59.3.0, DataFusion 55.1.0, object_store 0.13.2, PyO3 0.29.2; evidence locks agree. |
| `just codegen-check` | **0 differences**, all 3 schema targets. New generated control module is marked intent-to-add so the tracked-output invariant runs. Optional Ipopt bindgen remains deferred. |
| `just adr-lint` | **68 records / 31 register rows, 0 findings**. |
| `cargo clippy --locked -p pse-catalog --lib --features pse-relations/force-validate -- -D warnings` | **0 project Clippy findings** after correcting 4 existing local binding/resolution/operation lints and the new implementation's findings. The transitive future-incompatibility warning below remains. |
| `cargo clippy --locked -p pse-catalog --lib --test unified_delta_dml --test unified_delta_contracts --features pse-relations/force-validate -- -D warnings` | **0 project Clippy findings** for the final library and both focused test targets; transitive warning remains. |

The DataFusion/Delta skill capability scans were inspected. Strict schema matching is
intentional: these writes do not silently merge/evolve declared contracts. The private
physical-input adapter truthfully refuses pushed predicates. Effectful command output
cardinality remains unknown to optimizers, avoiding statistical elimination of a write.
These advisory scans are not runtime certification. The later session/rule capability
scan reports seven advisory matches: five provider/statistics hooks and two explicit
materialization sites. Complete-source capture cannot push filters past validation;
read-only buffers/candidates claim no predicate support; the deferred mutation target
is a DML boundary; effect cardinality stays unknown; the two collection sites are
explicit finite materialization. These dispositions do not close the old-operation
or old-provider deletion obligations.

The broad catalog/rules/compiler Clippy run failed with 66 findings (baseline 0),
primarily compiler function-size/argument-count findings. Its one introduced
`let_and_return` finding is fixed; the full command needs a fresh successful run.
The later catalog-only check found two new documentation/style findings in the
shared materialization change; both are fixed, and the final catalog/numerics check
above passes. The numerical capability-gap scan reports 0 advisory matches; that is
syntax evidence, not complete behavioral qualification.

**Tested:** `just py-test` passed **80 unit/component tests, 0 failures, 0 skips**
on Python 3.14.7, 32 pytest workers, baseline 0. Its fixture is still the predecessor
store, so this is a regression check, not target Delta/Python acceptance. `just quality`
passed its configured format/lint/types/import/repository/setup gates: 0 findings,
4 import contracts kept, 14 setup tests passed. Captured upstream skill content and
manifests are excluded from project formatters, including runtime alias paths; the
captured files were not reformatted. The ADR supersession union is type-correct.

**Earlier, before the source-text cut:** `just py-sync` rebuilt the editable native extension and regenerated its actual API
stub. `just doctor` then passed all checks. `uv` includes
Rust test sources in the editable build cache key, so its environment check can become
stale after test-only edits. That earlier `just codegen-check` reported 0 differences
across all 3 schema targets. A subsequent 2026-09-16 `just py-sync` also succeeded
after the source-text changes. Another `just py-sync` succeeded after native-expression
work, before the final scalar-math adapter/test changes; those later Rust edits require
a final refresh before current Python acceptance.

### Open implementation and verification

UD00–UD12 and M1–M5 remain open. Next work continues the UD01–UD03 functional cut:

1. Replace root-only `OperationNode` dispatch/fake children and connect actual product
   consumers to the target session and Delta relations. Delete the superseded store,
   snapshot, manifest/CAS, stage/context and publication-journal paths with callers.
2. Complete write-hook/schema-mutation coverage, domain/completeness admission, actual
   function/policy binding, resource/cancellation and post-commit-hook fault cases.
3. Complete source construction/inference and case/structure/numerics, then actual
   Ipopt/Pyomo/NL solves, cold Python inspection, CDF/reuse and protected maintenance.
4. Qualify all S/V cases, extension/cost cases and independent G1–G7 decisions.

Current publication reconciliation requires retained control logs and the selected
control-row files. Automatic control checkpoint/log cleanup is disabled; coordinated
reader/retention policy is still UD09 work. Local-file and in-memory backend probes do
not certify every remote backend. Native DML hooks now have focused tests; complete
product policy binding, schema/maintenance operations and opaque artifacts remain open.
Source text now persists in declared Delta rows; full source construction and normalization
through the target execution path remain open. Editable attempt providers expose the explicit durable
schema; published semantic providers remain read-only. Remaining old durable-store consumers
and their deletion obligations remain open.

**Tested — failing gate:** `just test-package pse-tests-conformance --test invariant_fixtures --no-fail-fast`
ran **2 tests: 1 passed, 1 failed, 0 skipped**, baseline 0, run
`e85ea20d-628f-41a0-b639-f06b4d4482c5`. The exact fixture inventory is missing
388 registered invariant directories, including the new publication key invariant;
no stale directories were found. Eight existing document fixtures now use source text.
The missing-fixture/terminal conformance obligation remains open; it is not waived.

`proc-macro-error2 2.0.1`, pulled by Delta's `validator_derive 0.19.0`, emits a Rust
future-incompatibility warning. It is not suppressed or accepted as a quality baseline.
The simulator recipe's Rust/Python targets are still absent and cannot pass. No full
workspace, numerical/backend, source-to-solve, retention or terminal acceptance is claimed.

### Native facts, common publication sessions and finite inference — 2026-09-16

**Implemented in part (UD02–UD05):**

- Rule inputs now own checked Arrow facts with an optional exact typed Delta member
  selection. `RuleInputLocation::Pinned`/`Stage` and snapshot-hash support fields are
  deleted. Generated support rows embed the declared publication-member type; transient
  facts explicitly carry no durable selection. Rule fixtures no longer create old stores.
- `SessionFactory::open_publication` binds lazy exact-version member views into the
  common `SnapshotSession`. Native view source dependencies survive inlining, so SQL,
  effect admission, scoped policy, cancellation and owned fact capture share one boundary.
  Captured facts retain their exact selection while releasing the opened publication.
- Typed `NativeExecutionContext` supplies the actual registry, caller configuration,
  function/planner bindings, scoped memory ceiling, cancellation and policy scopes to
  specialized physical algorithms. Rebinding executed child rows creates a private name
  scope with the same services. No default session or predecessor graph is installed.
- Finite rule inference now executes through `PseFixedPoint`/`PseFixedPointExec` in the
  composed native planner. All source and support mappings are real physical children.
  The iteration consumes executed child fields; changed source evidence is refused.
  Native round plans preserve four-valued decisions, support, negation and conflicts.
- The operator returns a transient typed Arrow tuple of declared result relations,
  derivations and round counts. Native projection/unnest consumes it; completed facts
  retain buffers only. It is not a universal durable model table. The separate public
  fixed-point execution loop is replaced by native preparation/execution.
- Conflict and exhaustion errors cross the physical stream as typed semantic diagnostics.
  Statistics remain unknown until admission/convergence; an optimizer cannot use an
  advertised exact count to remove required fixed-point work.

**Tested**, default profile, explicit `pse-relations/force-validate`, baseline 0:

| Command | Result and scope |
|---|---|
| `just test-package pse-rules --no-fail-fast` | **61 passed, 0 failed/skipped**, 8.331 s, run `6c013a5a-0109-4c9b-a114-b55ad2e0926d`. Includes native composition, actual changed-child refusal, input-owner release and preserved typed conflict/round-limit diagnostics. Includes concise native JSON EXPLAIN after the corrected consumer rerun. |
| `just test-package pse-catalog --lib native_algorithms_keep --no-fail-fast` | **1 passed, 0 failed, 112 filtered**, 0.669 s, run `430a0852-7b94-4cdc-bd1c-f3e736e54e80`. Actual custom function, required batch size, inherited schema effect policy, scoped allocation ceiling, foreign-session refusal and cancellation. |
| `just test-package pse-catalog --test provider_contracts --test captured_native_layouts --no-fail-fast` | **23 passed, 0 failed/skipped**, 0.209 s, run `6382d34b-b9bb-40a5-a0f6-f654808ea9b2`. Common provider policy, captures, read-only materializations and owned streams. |
| `just test-package pse-tests-engine --test unified_sources --no-fail-fast` | Earlier common-session source receipt: **1 passed, 0 failed/skipped**, 10.042 s, run `3bc438d2-d65e-488c-bf5f-514fd16d471d`. Exact selected documents become native support after the publication owner is dropped. |

**Tested — focused static check:**
`cargo clippy --locked -p pse-rules -p pse-catalog --lib --tests --features pse-relations/force-validate -- -D warnings`
has **0 project findings** after the concise-debug/statistics changes. The
transitive `proc-macro-error2 2.0.1` future-compatibility warning remains open.

**Mistake and correction:** the first CI-profile source-to-P10 consumer rerun failed
its unchanged 32 GiB resource budget during legacy terminal-record persistence. Native
JSON EXPLAIN uses extension `Debug`; the new node's derived implementation dumped its
captured facts/registry into diagnostics. The node now formats only structural rule
identity/limits, with no fact or registry dump. The corrected rerun, `just test-package pse-tests-engine --test unified_sources --test native_template_graph --profile ci --no-fail-fast`, passed **2 tests, 0 failed/skipped (1 slow)** in 261.120 s, run `66282916-fb6f-4a20-81ce-b98c25674f38`, CI profile/force-validation/baseline 0. The
Delta source case in the initially failed run also passed. This is not a solver or target compiler
acceptance receipt; the legacy driver/store and closed rule declaration algebra remain
to be replaced. No UD package, S/V case, milestone or G gate is closed by these checks.


**Implemented — UD07 solver binding prerequisite:** the Ipopt bindgen placeholder and
hygiene-only comparison are deleted. `just codegen --only bindgen` generates the actual
C API and callback types with pinned bindgen 0.73.2; `just codegen --check --only bindgen`
compares actual regeneration. Both digest-pinned solver-image extraction and an explicit
`IPOPT_DIR` prefix reproduce identical output (C header receipt
`14455434218739abfb1c56cb60cf99f98400b24d69feeb60f0e717349496b34b`). These are generation
checks, not solver execution. The common full `codegen-check` now includes real bindgen.

**Tested — broader static boundary remains failed:** adding `-p xtask` to the focused
catalog/rules Clippy command compiled its compiler dependency and reported **65 existing
compiler findings**, baseline 0. No compiler Clippy closure is claimed. The native
capability-gap scan reports **0 advisory matches** after the explicit unknown-statistics
contract; it does not certify semantics. `just family-check` passes after the dependency
edges were added.


**Mistake and correction — generator feature graph:** bindgen enables prettyplease's
`verbatim` macro formatting, so xtask initially emitted different bytes from standalone
schema tests. The workspace now explicitly enables that feature for the single pinned
formatter, and `just codegen` regenerated all targets. The generator no longer depends
on which consumer unified Cargo features. **Tested:** `just governance` completed with
59 passed, 0 failed, 0 skipped (default / force-validate; baseline 0), Nextest run
`95ef2607-1c64-476a-a8c8-4a6549aea6a4`, 3.012 seconds. All three schema targets and
the actual Ipopt bindings regenerate identically; dependency-family validation passes.
The focused xtask check,
`cargo clippy --locked -p xtask --no-default-features --all-targets --features pse-relations/force-validate -- -D warnings`,
has **0 findings** after the new generator and a pre-existing no-op semicolon were fixed.

### Native Ipopt execution and completion assessment — 2026-09-16

**Implemented in part (UD07):** an actual native `Solve` logical/physical operator
owns its numerical child, compiles expressions/Jacobian once, and invokes the real
Ipopt C API with explicit limited-memory Hessian policy. Mutable callback workspace,
panic containment, concurrency permits, cancellation and output ownership stay inside
native execution. The common execution context binds actual effective policy and
invocation cancellation; foreign work retains its owners until it returns.
`just native-solver-test` runs linked tests in the digest-pinned solver image.

**Tested, baseline 0:**

| Command | Mode | Result / receipt |
|---|---|---|
| `just native-solver-test --no-fail-fast` | default; Ipopt + force-validate; pinned-container target runner | **12 passed, 0 failed, 0 skipped**, 2.100 s; `7cb2ed4d-6c74-423f-a83e-7977ff4e32dc` |
| `just test-package pse-catalog --lib --no-fail-fast` | default / force-validate | **113 passed, 0 failed, 0 skipped**, 5.668 s; `ad64da8c-974c-44df-b21c-3b2a47e32c89` |
| `just test-package pse-backend-native --no-fail-fast` | default / force-validate; Ipopt off | **2 passed, 0 failed, 0 skipped**, 0.010 s; `7d5480ed-8f61-4077-80cb-d40c15969353`; before the last linked cancellation refinement |

The pinned solver image, tested scenarios and source hashes are recorded in the
[completion review](../design_review/reviews/design_review_unified-datafusion-delta-completion_2026-09-16.md)
and its evidence snapshot. These are component receipts. The solver is installed by
its tests, not by the product runtime; case/structure/initialization, generated
Pyomo/NL routes and durable run publication remain open. Its nonlinear mixer/heater
example is constructed numerical input, not the source-derived Slice A journey.
The foreign-memory allowance is budget admission, not a measured/enforced bound on
all C-library allocations. Full final quality and resource qualification remain open.

**Mistake and correction:** a relative target-runner path initially failed; a later
Cargo CLI runner configuration did not reach Nextest. The recipe now sets the target
runner through the environment with a repository-resolved absolute path, and linked
tests require the container solver prefix. The final 12-test receipt above used that
runner; an intermediate host-linked success is not pinned-container qualification.

**Assessment outcome:** the detailed review records 7 partial packages, 6 open packages,
no package or milestone closure, G1 failed for the target lifecycle and G2–G7 unresolved.
It maps every original F/V item and names the remaining live legacy caller closures.
The next implementation work is a target-only functional cut with predecessor deletion,
as specified near the start of this plan and in review §11. This assessment changed
documentation only; it did not implement those remaining cuts or rerun runtime suites.
