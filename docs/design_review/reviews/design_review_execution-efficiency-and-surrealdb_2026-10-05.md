# Execution efficiency and SurrealDB target design review

**Current disposition owner:** [Plan 28](../../plans/28-surrealdb-unified-substrate.md#finding-dispositions)
owns the adopted EF findings together with the unified-substrate follow-up findings.

**Follow-up:** the [unified simulation-substrate review](design_review_surrealdb-unified-simulation-substrate_2026-10-05.md)
reassesses EF01–EF08 and this review's architectural recommendation against a canonical
problem/dependency/run/result database with durable runs by default. The original observations
and evidence limits below are retained.

**Date:** 2026-10-05\
**Tier / purpose:** Comprehensive system design review / target\
**Standard:** Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator 1.5,
[pse-arrow binding](../design_principles/standard.toml)\
**Reviewer:** Design-reviewer contract, Sol/high. The reviewer previously performed the
bounded build investigation and then independently assessed the combined design. This is
independent of the coordinator's principal judgment, but is not fresh-context independence
on build evidence. The coordinator inspected decisive evidence and integrated the review.\
**Decision:** **Revise.** Architectural fitness fails G9. This is a design judgment, not a
new scientific qualification of the implementation.

## Assessment and recommended architecture decision

The simulator has sound foundations worth preserving: explicit physical meanings, authored
model authority, library-owned mathematics and numerical iteration, typed problem-class
selection, immutable preparation products, attempt-owned native state, and publication
separated from numerical execution. The review does not establish that Rust, Arrow,
DataFusion or PostgreSQL intrinsically cause the reported delays.

It does establish avoidable work at concrete boundaries. Successive source edits retain
complete ancestral bundles; immutable admission loses the context needed to reuse its
established guarantee; an individual durable dispatch loads and evaluates the complete
study; small semantic builds inherit execution and storage dependencies; whole-tree
provenance propagates unrelated edits through shared consumers; narrow build measurements
prepare broad native capabilities; and no-op generation rewrites unchanged sources. These
mechanisms warrant revision without waiting for a timing distribution. Their quantitative
contributions and the gains from correcting them remain unmeasured.

**Select correction of these ownership, preparation and dependency boundaries as the next
architecture direction. Do not select wholesale SurrealDB consolidation as the remedy for
the demonstrated efficiency defects.** This recommendation does not require preserving the
present store indefinitely.

SurrealDB is a credible preferred candidate over Neo4j for a future durable owner of
heterogeneous authored documents and identity-bearing relationships, where connected
selection and inspection are substantive requirements. Adoption of that role is
**unresolved**: the consequential decision is whether the durable store should own editable
declarations, derived graph views, operational state, published scientific results, or a
specified combination. That decision changes admission, revision, interoperability and
recovery contracts. It is a missing ownership premise, not merely a missing benchmark.

A database-centric compiler is also eligible. Select it around explicit inference rules and
complete operations that its native engine or a qualified shared kernel can own more
simply. Stored types, graph traversal and database functions do not alone replace contextual
physical inference, derivative preparation or numerical solving.

## Scope, baseline and evidence

The functional target is a local-first, extensible process simulator supporting ordinary
edit/re-solve, concurrent local studies, recycles, dynamics, optimization and fitting, with
truthful physical results and credible execution as structure, case count and concurrency
grow. Future shared services remain a legitimate variation axis.

Inspection began on `main` at `6498b013e579ec8039573200c92282eb8715b52e` alongside
concurrent Plan 27 numerical edits. The principal finding sources remained unchanged during
their examination; this review does not certify that concurrent work's completion.

No product builds, tests, database operations, benchmarks or cache changes were performed
for this review. Source and interface conclusions are **Interface-checked**; corrections are
**Proposed**. Historical measurements retain their original conditions.

Supporting evidence:

- [Source lifetimes and admission](../evidence/execution-efficiency-2026-10-05/source-lifetimes-and-admission.md)
- [Rust and native build turnaround](../evidence/execution-efficiency-2026-10-05/build-turnaround.md)
- [Runtime preparation and studies](../evidence/execution-efficiency-2026-10-05/runtime-preparation-and-studies.md)
- [SurrealDB and Neo4j capabilities](../evidence/execution-efficiency-2026-10-05/surrealdb-capabilities.md)

These documents supply bounded evidence. This principal review owns the combined judgment.
Publication completes the requested review; it does not authorize implementing its proposed
remedies or migrating storage. Current work remains at [its existing owner](../../plans/README.md).

## Responsibilities, contracts and scientific meaning

The consequential decomposition is appropriate at the semantic level:

| Owner | Hidden decision and consumed contract | State, effects and local reasoning |
|---|---|---|
| Authoring and modeling | Source identities, names, bindings, quantities, physical inference and finite specialization | Immutable document/model revisions; pure checks receive explicit semantic inputs |
| Compiler and preparation | Dependency-derived products, selected outputs, coordinates and implementation identities | Salsa and retained preparation; mutable solver workspaces stay outside pure queries |
| Mathematics and native adapters | Expressions, derivatives, structure, numerical methods and capability contracts | Library evaluators and attempt-compatible native state; adapters own ABI and settings translation |
| Workflow and studies | Intent, dependency policy, attempts, cancellation and outcome interpretation | Pure policy plus supervisors; durable adapters add leases and revision fences |
| Operations and publication | Claims, catalog visibility, retention and artifact coordination | PostgreSQL transactions and Delta/file effects have distinct authorities and failure scopes |

Adding a unit or property model should principally extend authored declarations and their
scientific behavior. Replacing a fitting solver should change capability selection and its
adapter. Changing result representation should affect persistence and conversion rather
than physical inference. A new database should absorb its value, transaction and lifecycle
details at those ownership boundaries.

Physical meaning must survive every proposed optimization:

| Meaning | Authority and enforcement | Required preservation |
|---|---|---|
| Dimensions, units, basis and conventions | Quantity and expression inference; typed model values | Database numeric types cannot substitute for quantity semantics |
| Components, phases, reference states and property applicability | Authored physical definitions and contextual admission | Reuse includes the actual interpretation/provider owner |
| Conservation and closure | Authored contributions and original-model qualification | Solver residuals alone do not establish physical closure |
| Topology, incidence and solve partition | Distinct typed relationships and structural analysis | Preserve isolates, direction, multiplicity, roles and scope |
| Domains, guards and smoothness | Formulation and evaluator contracts | Compaction and alternative evaluation retain guards and required dependencies |
| Outcome and usability | Declared intent, numerical evidence and independent assessment | Availability, termination, qualification and usability remain distinct |

The inspected well-posedness design checks original structural scope and names responsible
equations or variables when rejecting it. Structural matching is not numerical rank, and
flowsheet connectivity is not equation solvability. Root and optimization intents have
different obligations. Supported dynamics have a declared ODE/index-one DAE scope; this
review does not extend it to general higher-index or hybrid systems.

The numerical stages retain distinct responsibilities:

| Stage | Domain and structure | Derivatives, scaling and class | Outcome obligation |
|---|---|---|---|
| Specialization/preparation | Physical admission, finite dependencies, guards and selected outputs | Explicit coordinate/order demand and capability requirements | Structured rejection before unsupported execution |
| Root/NLP/discrete execution | Original model plus admitted formulation | Class-specific libraries own iteration, globalization and factors | Termination stays separate from original-model qualification |
| Recycles/dynamics/fitting | Declared strategy, initialization and mode assumptions | Shared model authority with mode-specific numerical requirements | New-state physical and numerical checks remain necessary |
| Publication/reopening | Retained scientific result and declared storage contract | No rerun of science during publication | Partial or uncertain effects cannot appear committed |

These are inspected architectural contracts, not newly executed scientific evidence.

## Representative scenarios

| ID | Scenario | Expected locality and observed concern |
|---|---|---|
| S01 | Repeatedly edit one document and re-solve | Retain current dependencies and intentionally held revisions; EF01/02 amplify live state/admission |
| S02 | Build/test a semantic owner after a body, API or unrelated backend edit | Recompile affected dependencies; EF04–07 broaden closure, invalidation or setup |
| S03 | Execute N dependent study points, including failures and retries | Reuse immutable structure and fence changing facts; EF03 repeats complete policy per point |
| S04 | Request a narrow derivative/output closure from a large body | Preparation follows required dependencies; EF08 leaves dense formal setup before demand |
| S05 | Add a model, replace a provider or compose a recycle/dynamic/fitting workflow | Preserve shared physical authority and library contracts; avoid copied scientific workflows |
| S06 | Change durable representation or interrupt publication | Persistence owns codecs and recovery; store replacement preserves claims, visibility and retention |
| S07 | Increase request diversity or concurrent native work | Retention and pools follow actual lifetimes; workspace rotation and eager study preparation need explicit tradeoffs |

## Findings

All corrections below are **Proposed** and unscheduled.

<a id="ef01"></a>
### EF01 — The latest source revision retains complete superseded bundles

**S01; AP-07, DP-19/20; G9.**

In [`OwnedDocumentSet::edit`](../../../crates/pse-runtime/src/authoring_driver/document/owned.rs),
an affected package copies its document inventory, reuses parser owners, and creates a
`BundleOwner` whose `_parent` retains the previous complete bundle. Each predecessor retains
its predecessor. Dropping external old-revision handles therefore does not release that
ancestry.

Preserving reused allocation ownership is necessary. Retaining every superseded bundle is
an unnecessarily broad ownership unit: live state grows with edit history independently of
the current model. This is not a claim that shared Arrow buffers are deeply copied.

Give immutable documents, parser products and columns directly retained owners and leases.
Revisions should retain actual dependencies, with history retained through explicit policy
or real consumers. Verify repeated edits with old handles dropped, then separately retain an
intentional old revision and check source access and lease lifetime.

<a id="ef02"></a>
### EF02 — Immutable local admission discards the native owner that established it

**S01/S05; AP-07, DP-03/09/10; G9.**

[`FieldCheckedBatch::admit`](../../../crates/pse-relations/src/columnar.rs) evaluates schema,
Arrow structure and local/native predicates, but retains storage and relation contract
without the actual validation-context owner.
[`validate_context`](../../../crates/pse-runtime/src/authoring_driver/document/owned.rs)
consequently re-admits unchanged batches when constructing physical and modeling contexts.
Predicate preparation is retained; successful evaluation of unchanged values is not.

Schema equality alone is insufficient: another native assembly can give it different
meaning. The engine already retains the actual immutable validation context, providing a
narrower sound reuse premise.

Retain local admission with unchanged storage, resolved contract and actual native owner.
Recheck changed data/context, imports, global closure and new numerical states. Keep explicit
force-validation controls. Verify same-owner reuse and reevaluation under a different native
predicate owner with identical schema. This is an ownership capability, not a new persisted
certificate system.

<a id="ef03"></a>
### EF03 — Individual durable dispatch evaluates the complete study

**S03; AP-07, DP-10/20; G9.**

[`Studies::admit_dispatch`](../../../crates/pse-operations/src/studies.rs) loads every point,
reconstructs policy facts, calls
[`study_policy::transition`](../../../crates/pse-operations/src/study_policy.rs), and selects
one requested action. Transition re-admits graph structure and derives actions for all
points. Workers invoke this for individual dispatches.

For N individually dispatched points, repeatedly processing N rows creates an N-squared
contribution before additional graph and recovery work. This is not a complexity claim for
the complete solve. Dependencies, current cancellation and claim fences are necessary;
unrelated action generation per dispatch is not.

Admit immutable graph structure once and evaluate the requested dependency closure against
fenced current facts, or admit a coherent batch of ready actions. Keep one policy authority
and preserve usable-predecessor semantics, cancellation, retries, warm-start selection and
stale-claim rejection. Differential policy cases and competing-claim tests should establish
equivalence; growing matched studies can then quantify the benefit.

<a id="ef04"></a>
### EF04 — Feature stabilization gives small semantic roots heavy execution dependencies

**S02/S05; AP-01/07, DP-17; G9.**

[`pse-model`](../../../crates/pse-model/Cargo.toml) depends on the generated workspace hack,
whose direct target and host dependencies include DataFusion execution/planning, Arrow,
cloud-enabled object storage and PostgreSQL support. A cold semantic build therefore compiles
machinery outside its consumed semantic contract.

The tradeoff is real: hakari and nightly workspace unification reduce duplicate feature
variants when recipes alternate. Existing exclusions and host/target separation are
strengths. Historical selected heavy-unit reduction does not settle the present small-root
tradeoff.

Retain the single family/type universe and intentional feature unification while narrowing
hack membership to genuine consumers. Compare focused semantic builds and alternating
composite recipes. Removing hakari everywhere is not the established correction.

<a id="ef05"></a>
### EF05 — Whole-tree provenance is an unnecessarily broad executable dependency

**S02; AP-01/07, DP-04/09/21; G9.**

[`pse-buildinfo/build.rs`](../../../crates/pse-buildinfo/build.rs) hashes files throughout
`crates/` and `vendor/`, including tests and unrelated implementations. Shared consumers
embed the changed identities. An unrelated source edit can therefore propagate compilation
beyond its implementation closure. Mathematical artifact requests also include both global
identities.

Complete dirty-source provenance is valuable, and unchanged-output writes are already
avoided. Its compilation placement and artifact-key breadth should be separate decisions.

Keep whole-build attestation at the executable/publication boundary. Preparation identities
should include complete relevant implementation, ABI, provider, configuration and toolchain
dependencies, conservatively where necessary. Never replace dirty-source identity with Git
HEAD.

The compiled-program cache is **process-local**. A new executable already loses its resident
cache; this finding does not establish additional cross-build eviction or durable-reuse
latency. Verify unrelated-edit locality and required evaluator/native invalidation separately.

<a id="ef06"></a>
### EF06 — Narrow build measurement prepares unrelated native capabilities first

**S02; AP-07, DP-10/20; G9.**

The default [`bench-builds`](../../../justfile) path sources solver/math environments before
selecting compiler/relations measurements. Native math setup prepares KLU and interval/root
infrastructure and reaches Uno/PETSc preparation. Uno identity construction invokes Cargo
to discover the HiGHS archive before checking installation freshness. Native cache hits
rehash installed headers/libraries under an exclusive per-identity lock.

Validated staged installation and explicit-prefix bypasses are strengths. The avoidable cause
is preparation broader than the requested capability closure, plus repeated verification
without an appropriate retained immutable lifetime. This does not establish that every
recipe always performs every native preparation.

Select native capabilities from the operation. Retain complete setup for composite/native
qualification and corruption detection at changed trust boundaries. Verify fresh-shell setup
plus Cargo, alongside Cargo-only timing; replacing validation with existence checks would
be unsound.

<a id="ef07"></a>
### EF07 — No-op generation rewrites unchanged sources

**S02; AP-07, DP-10; G9.**

[`write_tree`](../../../xtask/src/codegen.rs), bindings generation and Python stub generation
write outputs without comparing existing bytes. Identical regeneration changes mtimes and
makes build systems reconsider freshness. Its latency effect is unmeasured.

Scoped generation, check-mode byte comparison and stale-output deletion already provide
useful boundaries. Preserve unchanged bytes and mtimes, write changed/new outputs, and retain
stale-file removal. Verify no-op generation and an authoritative declaration edit. Correct
the generator rather than its generated products.

<a id="ef08"></a>
### EF08 — Narrow symbolic demand still begins with dense formal-slot setup

**S04; AP-07, DP-10; local architectural judgment unresolved.**

[`support_with_allowance` and `compile_scope`](../../../crates/pse-math/src/execution.rs)
create full-body parameter/symbol structures and dense facts or reachability storage before
completing selected demand analysis. Output/stage pruning exists; the evidence does not show
that all arithmetic or derivatives are compiled.

For disconnected or narrowly demanded structure, this setup can exceed the required closure.
The unresolved premise is whether a consumed evaluator/library contract requires the full
formal signature.

Determine that contract and compare compact library-local coordinates with dense preparation,
preserving public coordinate meaning, guards, branches and provider dependencies. Dense
workloads may favor the present layout. Declared large preparation limits are workload
premises, not measured capacity.

## Additional runtime and numerical alternatives

Whole ephemeral studies prepare every point before scheduling. That preserves complete
admission before effects, and compatible products/value bindings are reused. It is therefore
not evidence that every point recompiles the model. Separate pure study admission from
expensive ready-attempt preparation, or bulk-bind compatible cases, only while preserving
the promised failure boundary.

Compiler retention first applies Salsa LRU limits and may then rebuild a fresh database.
This is lawful conservative reclamation with a possible reuse cliff under request diversity.
A/B/A and span-only reuse tests contradict universal invalidation. Selective retention is a
candidate, not a diagnosed production bottleneck.

The mathematics already uses Symbolica multi-output evaluators, common-expression
optimization and HyperDual vectorization. That vectorization is derivative algebra, not
data-parallel SIMD. The artifact ABI explicitly identifies interpreted evaluation without
JIT/data SIMD. Symbolica exposes
[native-code-generation alternatives](https://symbolica.io/docs/numerical_evaluation.html),
but current features omit that capability. JIT, SIMD or batch evaluation may suit stable
expressions reused across iterations/cases; cold preparation, native-code lifetime, platform
behavior and numerical parity remain part of the complete operation. The lock resolves
Symbolica 3.0.1 while the shared skill's source is 3.0.0; consumer-version qualification is
required before adoption. Such evaluation cannot itself remove the earlier dense support
setup in EF08.

Publication already provides counterevidence to a blanket eager-materialization diagnosis:
member metadata opening is bounded, rows remain lazy, native views retain pushdowns, and
writers can stream. Necessary dependency materialization avoids execution deadlock. Assess
particular crossings rather than replacing Arrow/DataFusion on the assumption that they
always materialize the whole result.

Required branch/root certification, derivative accuracy, physical checks and original-model
qualification are not waste merely because they are expensive.

## SurrealDB, Neo4j and the relationally compiled type system

### Where SurrealDB fits

The [version-scoped capability investigation](../evidence/execution-efficiency-2026-10-05/surrealdb-capabilities.md)
examines the neo4j-surrealdb skill, exact SurrealDB 3.3.0 source, current primary documentation,
Context7 and existing probes. Its findings distinguish documentation, implementation and
small executed historical probes; none supplies product-scale performance evidence.

Nested documents, typed record identities, required/optional fields, literal unions,
assertions, unique indexes, identity-bearing relation records, connected queries, functions
and multiple local deployment modes substantively fit heterogeneous authored knowledge.
Generated schema and codecs can lower one authoritative declaration without creating a
second editable model.

There are three distinct opportunities: lower declaration types into storage schema;
perform indexed structural resolution and closure; execute contextual inference and finite
specialization. The first two have direct library support. The third needs explicit
scientific rules and their fixed-point, absence, deletion and revision semantics.
Structural schema enforcement does not establish dimensions, basis, reference state,
property applicability, finite specialization or conservation. A relationally compiled type
system is nevertheless a legitimate alternative: explicit inference rules could lower to
database set/fixed-point operations or qualified shared kernels. Preserve missing-name
dependencies, substitutions, cycles, selected revisions and source diagnostics. A transitive
reference query is only part of that operation.

Useful native queries include selected package closure, eligibility, dependents, lineage and
source-linked result explanation. The exact optimizer has indexed single-hop graph-semijoin
and pushdown opportunities; that is stronger evidence than syntax alone, but not an arbitrary
multi-hop optimization guarantee. Coarse selected queries should preserve predicates,
projection and bounds rather than induce host-driven one-record hydration.

Classic relation records can preserve parallel assertion identities and participate as
endpoints. N-ary assertions and ordered operands need explicit roles/reification. Topology,
semantic dependency, equation incidence and solve order remain different graphs. Neo4j GDS
or SurrealDB traversal cannot erase those distinctions.

### What transfers from library-context

The linked library-context reviews are useful examples of selecting graph-native execution
and interrogating real database capabilities. Their lesson transfers: choose ownership and
complete operations first, expose native selection/bulk opportunities, and remove redundant
representations instead of merely replacing a driver.

Their database decision does not transfer automatically. This repository already accepts a
typed `modeling_package`, retains checked source exports and tracked document inputs, and
compiles in memory. PostgreSQL supplies operational transactions rather than the compiler's
relational scratch space. Disposable data, single-operator assumptions and reconstruction
policies must be selected for this simulator's documents/results independently. The linked
reviews and their precise limits are recorded in the supporting capability investigation.

### Integration and recovery obligations

The value boundary needs a lossless scientific codec. Historical probes identify distinct
`RecordId` versus string and `NONE` versus `NULL`, signed-zero loss over ws/http, and wrapping
of bound unsigned integers beyond signed range. Float bits and numeric projections may need
separate representations. Keep these details in the adapter rather than exposing database
values throughout mathematics.

Transactions support bounded changes, but current PostgreSQL queue claiming with ordered
`FOR UPDATE SKIP LOCKED` is not a direct translation. Inspected native backends permit write
skew unless relevant decision records enter conflict checking; record-ID locking does not
automatically protect predicate membership or absence reads. Claims, leases, retries,
concurrent inserts and retention need a specific protocol.

Every statement result must be checked. Imports can bypass ordinary checks, and LIVE streams
require cleanup/reconnection. Notifications remain hints followed by authoritative rereads.
If Delta/files remain external, publication intents, uncertain-commit settlement and cleanup
survive database replacement. Logical export/reopen probes do not qualify interrupted
publication or power-loss recovery.

Precomputed views and LIVE updates do not automatically replace Salsa dependency tracking.
Linked-table changes can leave views stale, and imports require view rebuilding.
Experimental Surrealism/WASM modules offer an eligible route for selected portable kernels,
but no inspected route qualifies the present Symbolica/native-solver graph. Avoid introducing
a second scientific interpreter merely to consolidate storage.

### Complete alternatives

| Alternative | Architectural benefit | Remaining cost and decision |
|---|---|---|
| Correct existing composition | Removes demonstrated retention, repeated admission/policy and build coupling at their owners | Retains current stores; selected next direction |
| Scoped SurrealDB graph/document owner | Native connected selection and nested typed persistence can replace specific lookup/hydration machinery | Requires declaration/view ownership, codec, revision and recovery decisions; preferred candidate for this role, adoption unresolved |
| Broad SurrealDB consolidation | Can remove PostgreSQL tooling and selected catalog/publication machinery when all adopted content fits | Arrow/SQL/file consumers prevent assuming wholesale deletion; operational protocols need redesign |
| Database-centric compiler | Can fuse particular rule/selection operations and reduce transfers | Full physical inference/finite closure contract and compatible kernel required; not selected wholesale |
| Neo4j graph owner/projection | Rich Cypher and established GDS are valuable for graph-centric analysis | Nested scientific documents require encoding/reification; projections retain state; Rust integration not qualified |

Local-first does not require embedding. A local SurrealDB server with the remote Rust SDK
separates database builds/resources/failure from the application; SDK defaults avoid the
embedded core. Embedded SurrealKV supplies persistent pure-Rust storage but adds the engine
closure and shares the application's resource envelope. RocksDB adds native C++/binding
work. Engine placement must avoid spreading this closure through the same foundation hack
diagnosed in EF04.

Neo4j remains technically eligible. Its GDS probes establish useful algorithms on tiny
graphs, not product-scale speed. Community and Enterprise constraints differ, and the
inspected Python route does not settle Rust integration. No license excludes either
candidate. Future search, vector, API and graph capabilities remain eligible without a
current consumer; assess their relevance and exactness obligations independently.

## Foundation and gate judgments

| Foundation | Judgment | Basis |
|---|---|---|
| AP-01 | **Violated** | EF04/05 couple independently changing semantic, execution and provenance owners |
| AP-02 | **Satisfied in inspected contracts** | Scientific and backend roles are distinct; proposed store details belong at persistence boundaries |
| AP-03 | **Satisfied in inspected composition** | Common model/preparation serves workflows; pure policy is separate from durable effects |
| AP-04 | **Satisfied at inspected architectural scope** | Physical definitions govern specialization, routing and interpretation |
| AP-05 | **Satisfied in inspected design** | Quantities, guards, dependencies, problem classes and outcomes are explicit |
| AP-06 | **Satisfied in inspected semantic owners** | Pure policy/model operations admit local tests; broad physical build closure remains EF04 |
| AP-07 | **Violated** | EF01–07 demonstrate unjustified work, state, invalidation or coordination amplification |

| Gate | Judgment and boundary |
|---|---|
| G1 Authority | **Pass:** no competing mutable scientific authority demonstrated |
| G2 Semantic fidelity | **Pass for inspected current design:** required distinctions are explicit; a future database codec remains unqualified |
| G3 Validity | **Pass for inspected admission design:** schema/contextual admission and structured refusal guard consumed validity premises |
| G4 Hidden behavior | **Pass for inspected stable paths:** preparation and effects have explicit owners |
| G5 Consistency/recovery | **Pass at inspected design level:** attempts, publication and cancellation are distinct; a replacement store protocol is unresolved |
| G6 Transformation/reuse | **Pass for inspected current contracts:** conservative keys/rechecks preserve premises, though overbroad; proposed shortcuts need verification |
| G7 Capability claims | **Pass:** unsupported numerical/store alternatives are not presented as qualified capabilities |
| G8 Library leverage | **Pass:** established libraries own mathematics and numerical iteration; no generic replacement requirement established |
| G9 Architectural fitness | **Fail:** AP-01/AP-07 violations |
| PS-G1 Physical consistency | **Pass for inspected design:** quantities, conventions and contextual physical admission remain explicit authorities |
| PS-G2 Well-posedness | **Pass for inspected supported routing design:** original structure, declared class and attributed refusal are explicit; matching is not numerical-rank proof |
| PS-G3 Numerical integrity | **Pass for inspected design:** class-specific execution, numerical assessment and original-model qualification remain distinct obligations |

These passes assess the examined architecture, not every implementation path or a new
numerical qualification. No rerun is needed merely to recognize the declared mechanisms;
their current implementation correctness and full scientific coverage remain with the
existing qualification work. SurrealDB replacement cannot inherit these passes without its
specific semantic and recovery contracts. EF08 remains unresolved at its narrower boundary.

## Measurements, model validation and limitations

Historical Plan 15 **Measured** evidence, recovered from commit
`8e9c3ce8264010e6cc2824161e7abb1b791d98c6`, reports stable 1.98.1, dev profile, 16 jobs
and explicit force validation:

| Operation | Uncached seconds | Cached seconds |
|---|---:|---:|
| Cold selected target/cache | 163.48 | 194.21 |
| Unchanged, median of three | 0.177 | 0.164 |
| Private compiler edit, median of three | 0.608 | 0.659 |
| Public compiler edit, median of three | 0.615 | 0.669 |
| Same-path artifact recovery | 164.21 | 28.48 |

Named raw reports were not found in the current inventory. These measurements predate the
current toolchain, profiles, unification and source; they do not establish present native-edit
cost. The [build evidence](../evidence/execution-efficiency-2026-10-05/build-turnaround.md)
records the historical source and conditions.

Current build instrumentation provides useful snapshots, freshness checks, Cargo timings and
cache deltas. Its native option retains Rust compiler edit stimuli; native C++/header,
generator and production-without-validation cases remain absent. Initial native setup
precedes its Cargo timer. The changed second-worktree target environment confounds ordinary
cross-checkout cache interpretation.

Plan 25k records untimed multi-minute preparation observations and an incomplete campaign;
these do not attribute delay to a finding. Existing K4 selectors can distinguish preparation,
reuse, studies and retention without a new instrumentation framework.

No unit/property family was numerically modified by this review. Their conformance remains
at the existing qualification owner and its exclusions. The IDAES parity oracle establishes
selected comparisons under its conditions, not universal numerical equivalence.

## Relationships, priority and next decision

EF01 and EF02 share immutable input lifetimes but concern different obligations: retaining
actual memory owners and retaining the guarantee established by a particular native owner.
One does not close the other. EF03 can change policy evaluation independently of the
database engine. EF04–07 concern separate causes of build cost and should not be collapsed
into a cache configuration change. A new database dependency could worsen EF04; a blanket
provenance removal could make EF05 faster while violating G6/G7.

The largest qualitative growth risks are ancestral source retention and per-point whole-study
dispatch. Build coupling independently affects ordinary development. This is consequence
priority, not a measured ranking of elapsed time. No-op generation is a narrow correction;
admission reuse first needs the actual-owner premise, and compact symbolic preparation first
needs the evaluator signature decision. The database ownership decision is a prerequisite to
a migration recommendation, not a prerequisite to correcting the established defects.

The remedies fit together if they retain one scientific authority, explicit immutable owners,
complete relevant dependencies, one study policy and attempt-scoped mutable state. Existing
old-revision consumers, changed native predicates, newly ready/cancelled study points,
required full evaluator signatures and interrupted publication are the legitimate cases
that constrain apparently simpler replacements.

## Rule impacts and follow-up ownership

Recommendations do not amend accepted rules themselves. The rule-impact decisions below
belong in subsequent authorized plan creation; that plan will own dispositions. This review
must not become a parallel implementation backlog.

| Anchor | Current rule or design choice | Dependent recommendation and proposed change | Recommendation if retained |
|---|---|---|---|
| <a id="rc01"></a>RC01 | **ADR-0122, Outcome 5 and Consequences**; `.config/hakari.toml`; AGENTS.md invariant “One feature set per dependency, whatever `-p` selects.” Hakari supplies broad feature stabilization, explicitly widening small members' closures. | **EF04:** narrow hack membership while retaining the single family/type universe and intentional workspace feature stabilization. Amend membership, not the safety of separate checkout targets. | EF04 remains a tradeoff requiring explicit disposition; caching does not remove the cold dependency closure. EF05–07 remain independently actionable. |
| <a id="rc02"></a>RC02 | **Blueprint §5.3 identity table and §20.4 lifetime table**, in `identity-and-publication.md`, include source/build identity in prepared-product reuse; **D14 and §14.4** require complete declared dependencies. Current realization is `pse-buildinfo/build.rs` plus `artifact_requests` in `pse-compiler/src/workspace.rs`. | **EF05:** distinguish whole-build attestation from relevant implementation identity and change compilation placement. Amend the specific identity/frame contract only if key inputs change; preserve D14's completeness requirement. | Keep complete source/build identity in keys, but supply it from the outer composition boundary rather than compiling a changing whole-tree constant into lower consumers. Key breadth remains conservative; no cross-build process-local cache penalty is claimed. |
| <a id="rc03"></a>RC03 | **`pse-operations/src/study_policy.rs`, documentation of `transition`:** exact snapshot, one action per authored occurrence; applying adapters reread/recompute under locks and check revisions. `Studies::admit_dispatch` realizes this separately for each point. | **EF03:** permit targeted dependency-closure admission under equivalent current-state fences, or coherent multi-action admission. Preserve shared policy authority and concurrency semantics. | Retain complete-snapshot transition and batch compatible ready actions under one coherent reread/recomputation instead of invoking it for each point. |
| <a id="rc04"></a>RC04 | **`justfile`, `bench-builds` recipe**, unconditionally sources `native-solver-env.sh` and `native-math-env.sh` before target selection. This is an implementation choice; no binding rule requiring broad setup for nonnative measurement was identified. | **EF06:** select requested capability closure and preserve full setup for composite qualification. This ordinarily needs a tooling change, not supersession of a foundational decision. | If full setup is intentionally retained, include it in complete-operation reporting and retain EF06's unnecessary-work disposition. A Cargo-only timer cannot represent fresh-shell turnaround. |
| <a id="rc05"></a>RC05 | **D10; blueprint §20 and §20.6; ADR-0114:** PostgreSQL owns operational/catalog state and Delta owns immutable published members. **D1 and §4:** the registry declares public/durable contracts and authored definitions are model authority. | **Conditional SurrealDB adoption:** change D10/§20/ADR-0114 only for responsibilities actually transferred. Change D1/§4 only if declaration authority or its lowering changes. Change §5.3 frames only where representation or interpretation affects identity. | Keep existing canonical operational/publication owners. SurrealDB may serve a derived graph/query role under the same authority if its added projection and lifecycle machinery is justified. No authoritative store migration follows. |

EF01, EF02 and EF07 principally change implementation ownership within existing meanings.
EF08 and the conditional runtime alternatives need their stated contract decisions before
implementation selection. All proposed owners remain **unscheduled**.

The next decision should adopt the concrete efficiency corrections and settle durable
declaration/result ownership before choosing a database migration scope. Acceptance requires
corrected architecture and appropriate semantic evidence, not a favorable aggregate timing
alone.
