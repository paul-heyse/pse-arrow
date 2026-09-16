---
title: Plan 07 completion and remaining DataFusion/Delta pivot
date: 2026-09-16
status: revise
scope: Plan 07 implementation and reachable product paths
---

# Plan 07 completion and remaining DataFusion/Delta pivot

## 1. Decision and scope

**Decision: Revise — implementation incomplete.** The target in
[Plan 07](../../plans/07-unified-datafusion-delta-hard-pivot.md) and the
[original review](design_review_unified-datafusion-delta_2026-09-15.md) remains binding.
This assessment does not propose a transition period or preservation of predecessor
objects. It identifies what must still be replaced and deleted to achieve that target.

**Completion:** 0 of 13 work packages are closed; 7 have partial implementation
(UD00–UD05, UD07), and 6 remain open (UD06, UD08–UD12). None of M1–M5 or G1–G7 is
closed. These are exit assessments, not percentages of effort or code volume.

**The central gap is integration and replacement.** Native Delta writes/publication,
generated durable layouts, owned fact providers, finite inference, numerical
evaluation and a real Ipopt operator exist. However, the compiler still operates
through the custom snapshot/store/stage/memo lifecycle, Python opens that store,
and source-derived cases do not reach the new solver. Adding native components has
not yet removed the predecessor product architecture.

**Method and coverage — Interface-checked / Implemented:** reviewed the plan's
packages, deletion ledger, milestones and verification requirements against actual
compiler, authoring, catalog/session, Delta, rule, numerics, backend, runtime, Python
and acceptance-tool callers. Source references below identify executable expressions,
not just names. Reviewed existing focused test logs from this implementation session;
§9 distinguishes their strength and source-state limits. The accompanying
[evidence snapshot](../evidence/unified-datafusion-delta-completion-2026-09-16.json)
records selected source hashes and receipt metadata for this dirty working tree.

This was a documentation-only completion review: no new runtime fault injection,
whole-workspace build, Python rebuild, remote-storage qualification or performance
campaign was run for the assessment. Static counterexamples identify reachable
paths; unexecuted failure scenarios remain proof obligations. No absence claim is
based solely on a symbol count. External services such as Unity are eligible future
integrations, not local simulator completion prerequisites. Exact-pin library
capability research from the original review remains the implementation basis;
published Delta 0.32 examples are not substituted for the pinned development source.

### Package completion matrix

| Package | Current evidence | Missing exit / required cut |
|---|---|---|
| UD00 — target and acceptance | **Implemented in part:** ADR-0068/blueprint revision 39, coverage/oracle contract, terminal command shell | Complete target fixtures and executable acceptance targets; finish target fixture/governance expectations. Proposed ADR status is not itself an implementation blocker. |
| UD01 — runtime and libraries | **Implemented in part:** combined Delta/PSE dependencies, composed planner, shared resources, real child plans in new extensions | Remove root `OperationNode` dispatch; preserve actual native configuration; integrate all product planners and effect admission; qualify nested paths and current full feature graph. |
| UD02 — schemas/providers | **Implemented in part:** generated durable layouts, exact providers, declared checks, owned captures, common publication-session opening | Close raw execution paths, domain/completeness admission, all provider-level policy/metadata coverage and full value/meaning round trips. |
| UD03 — Delta lifecycle | **Implemented in part:** validating writes/DML, exact member versions, conditional control publication and control retry reconciliation | Replace all old store callers; complete composed-write retry identity, candidate completeness, member reuse/slices, cold OS-process opening and failure matrix. **The deletion exit is unmet.** |
| UD04 — authoring | **Implemented in part:** exact document rows, grouped native parser, source publication/reparse from Delta | Connect edits/renames/normalization and all heater/mixer × FTPx/FcTP sources to the target lifecycle; remove old candidate/change persistence and source orchestration where replaced. |
| UD05 — inference/math | **Implemented in part:** native fixed-point operator with executed children, native rule lowering, typed support and owned facts | Delete closed relational rule/type/trace interpretation and procedural relational assembly; source-derived target topology/property/math/support coverage remains incomplete. |
| UD06 — problem/initialization | **Open:** structural/plans crate boundaries remain stubs | Implement case overlay, index/discretization lowering, fixed/free/bounds/objective ordering, incidence/DOF/blocks, scaling and initialization as native plans/operators. |
| UD07 — native solve | **Implemented in part / bounded Tested:** compiled expressions/Jacobian, real native Ipopt operator, failures/cancellation/ownership | Install planner in product runtime, consume UD06 problems, complete kernel/derivative coverage, publish durable run/results and qualify source-derived scientific cases and foreign-resource envelope. |
| UD08 — Pyomo and NL/SOL | **Open:** backend crates remain stubs | Generate both routes from the same problem contract; implement native export/invocation/ingestion and actual supported backend execution. |
| UD09 — change/reuse/retention | **Open:** old artifact memo and stage restoration remain active | Native semantic dependency/diff/reuse queries and Delta CDF; live publication/run/reader pins; qualified compaction, data-file and log retention; delete old memo/sidecars. |
| UD10 — Rust/SQL/Python | **Open:** old Python store/manifest handles and snapshot diagnostic codec remain | Same exact Delta publication/run through all interfaces; owned Python streams; supported target descriptors and fresh physical planning; delete old handles/codecs. |
| UD11 — qualification/cost | **Open:** useful focused tests exist | Implement missing terminal targets; full Slice A, ordinary extension and adversarial journeys; independent physical oracles and measured small/scaled costs. |
| UD12 — deletion/closure | **Open:** predecessor runtime mechanisms are reachable | Finish deletion in each owning cut, then audit absence and run final current-source Rust/Python/generation/governance/docs and independent G1–G7 review. |

### Functional outcomes, rather than predecessor equivalence

No S01–S11 outcome is closed for its full stated scope. In particular, a source
round trip, P10 snapshot or successful numerical example does not substitute for
these process-simulator outcomes.

| Outcome | Present boundary | Required completion |
|---|---|---|
| S01 author/change | Source text and parsing publish to Delta; edits use separate candidate machinery | Target-native edits/renames/cases and identity/source diagnostics |
| S02 construct process model | Existing compiler derives useful facts through the old lifecycle | Native target topology, property selection, contributions and balances from all required source cases |
| S03 mathematical meaning | Typed facts, support and supported scalar lowering exist | Full quantity/index/guard/implicit/kernel meaning and derivative requirements across lowerings |
| S04 solvable problem | No complete UD06 path | Case-bound ordering/bounds/objectives, incidence/DOF, scaling and initialization |
| S05 native simulation | Real constructed-expression Ipopt execution | Source-derived problem and initialized flowsheet through product runtime, with structured outcomes |
| S06 backend routes | Pyomo/NL crates are unimplemented boundaries | Actual generated Pyomo and NL/SOL routes from the shared problem |
| S07 publish/reopen | Exact Delta source/member/control components work | Complete model/case/run, composed recovery and cold OS-process reads without old objects |
| S08 change/reuse/maintain | Old memo/change infrastructure remains | Target dependencies, qualified reuse/CDF and protected data/log cleanup |
| S09 inspect/integrate | Native providers/streams exist; Python opens old stores | Same admitted target facts/results/settings through Rust, SQL and Python |
| S10 extend through contracts | Native registration and generation are foundations | Ordinary method/invariant/provider extension without new independent semantic dispatch |
| S11 failures/resources | Several native numerical and publication boundaries are tested | Complete product distinctions, durable effects/recovery and measured/enforced resource envelope |

## 2. Authority and lifecycle map

| Meaning | Target authority/boundary | Current implementation | Remaining replacement |
|---|---|---|---|
| Contract/type/invariant | One declaration, generated Arrow/Delta fields and native checks | Durable layouts/check bindings exist; rule relational typing also remains independently interpreted | Retain domain meaning once; use native expression typing and generated domain obligations |
| Source/model/case revision | Typed Delta publication selecting exact members | Delta source publication exists beside custom `Catalog`/`Snapshot` publication | One publication basis for authoring, compilation, inspection and runs |
| Compiled mathematics | Typed relations; native transforms; derived algorithm layouts | Native facts and useful numerical layouts coexist with stage snapshots and predecessor traversal | Remove persistent graph/stage authority; retain only layouts with actual target consumers |
| Execution policy | Actual runtime/session/provider bindings | Common policy boundary exists; public raw publication reads and concrete extension roster remain | One admission path and complete native operation effect contract |
| Solve attempt/result | Native execution workspace; typed terminal output; separate Delta publication outcome | Real Ipopt workspace/output; no product source-to-run publication | Case-to-solver integration and durable settings/input/result/diagnostic relations |
| Reuse/retention | Exact semantic dependencies and live Delta references | Artifact memo, durable hints, old contexts; no complete target retention path | Native dependency/CDF queries and coordinated reader/cleanup protocol |
| Python observation | Same admitted publication, owned Arrow stream | `open_store` → old local catalog; `head`/snapshot use old ref/manifest identities | Target publication/run handles; no compatibility reader |

Two storage mechanisms do not become one authority merely because both have native
provider wrappers. `Driver::run` writes old stage hints and stores old snapshots
(`crates/pse-compiler/src/driver.rs:139`, `:155`), while `DeltaPublish` publishes a
separate exact-version vector. There is no single product publication protocol
reconciling these roots. The required correction is deletion of the old lifecycle.

## 3. Contracts and invariants still to close

1. **A publication must mean a complete declared product state.** Current candidate
   checks iterate supplied members, establish local values/keys, top-level foreign
   keys and source spans (`crates/pse-catalog/src/delta/admission.rs:38`, `:48`, `:62`).
   They do not establish that a model/case/run contains every relation, demand,
   equation, initialization result or outcome required by its publication kind.
   Add generated domain/completeness violation plans over the exact candidate vector.
   A well-typed source publication is not a complete compiled process model.
2. **Every execution route must carry actual common policy.**
   `Publication::relation_stream` creates a context and calls physical planning and
   `execute_stream` directly (`delta/publication.rs:107`, `:118`, `:131` under
   `crates/pse-catalog/src`). It does not call common PSE preparation. The newer
   `SessionFactory::open_publication` route does not make this public alternative
   disappear. Restrict low-level assembly helpers and route product reads through
   common admission; test policy refusals through Rust, SQL and Python.
3. **Native settings must survive native assembly deliberately.**
   `SessionFactory::from_builder` constructs a fresh config and applies
   `.with_config(config)` (`crates/pse-catalog/src/session/factory.rs:89`, `:98`).
   Caller-supplied native config/extensions are replaced. Specify which PSE settings
   override native values, preserve unrelated native settings/extensions and prove
   this using an actual sentinel extension, not just a UDF-name check.
4. **Retry identity covers candidate effects, not only the root row.** Complete-row
   root reconciliation is useful, but `publication_plan::plan` creates write children
   before root publication and records `pse.attempt` as commit metadata
   (`crates/pse-catalog/src/delta/publication_plan.rs:79`). This is not an executed
   replay/reconciliation protocol for already committed member writes. Qualify failure
   after each child and the root, same-input retry and changed-input rejection.
5. **Exact selection is broader than the current composition helper.** The helper
   requires an empty member header and actual writes, and emits no revision selector
   (`publication_plan.rs:48`, `:71`). Exact selection exists on reads, but composed
   unchanged-member reuse and revision-slice publication are not complete. Reuse
   prior exact member descriptors where valid; do not duplicate the publication schema.
6. **Native numerical success is separate from simulator correctness and durability.**
   The solver accepts a constructed numerical child. It does not construct cases,
   prove process-model completeness or commit a run. Preserve explicit numerical
   failure/cancellation and publication outcomes when these operations are composed.

## 4. Derivation and execution: what still runs outside the target basis

### Required deletion ledger — live callers, not historical names

| Surviving mechanism | Actual reachable evidence | Native/Delta replacement and deletion boundary |
|---|---|---|
| Custom durable store | `crates/pse-catalog/src/store/layout.rs:24`–`:41`: refs, manifests, relation artifacts, evidence, stages, changes, contexts and JSON suffix | Delta tables plus typed publication/attempt/run data. Delete store layouts/control/membership/encoding modules and all callers in UD03; do not wait for UD12. |
| Snapshot/pass controller and stage memo | `crates/pse-compiler/src/driver.rs:62`, `:98`, `:139`, `:155`; `driver/execution.rs:208`, `:218`: prepare/publish old productions | Native model/case plans over exact providers, explicit useful Delta checkpoints. Remove `Driver`/`PipelineRequest` snapshot contract, old stage publication/restoration and artifact memo consumers. |
| Root operation callback envelope | `crates/pse-catalog/src/session/preparation.rs:332`; `session/operation.rs:245`, `:273`: no physical children, captured callback execution | Normal composed extension planning with real children; retain opaque native leaves only for actual specialized algorithms/effects. Delete generic legacy operation dispatch after its callers are cut over. |
| Closed relational rule algebra | `crates/pse-schema/src/model/rule.rs:346`; `model/rule_validation.rs:176`; `crates/pse-rules/src/plan/lower.rs:27`, `plan/trace.rs:100` | Native `LogicalPlan`/`Expr` for relational/type/operator behavior; domain truth, support, absence and termination remain explicit data/contracts. Delete duplicate relational interpreters and dispatch arms. |
| Separate change candidate ownership | `crates/pse-authoring/src/change_set/apply.rs:39`, `:86`: private execution, mutable relation map, `OwnedCandidateSnapshot` result | Preserve useful native change/rename queries and exact before-image semantics, but feed target candidate plans and Delta commands directly. Remove predecessor candidate/completion/persistence envelopes when callers move. |
| Old Python opening and identity | `crates/pse-py/src/inspection/handles.rs:88`, `:105`, `:249`: pinned old ref, `ManifestRef`, local old catalog | Exact Delta publication/run opening through shared runtime/policy and owned Arrow outputs. Delete old public handles and digest-based opening contracts. |
| Snapshot-bound diagnostic reconstruction | `crates/pse-catalog/src/session/plan_codec.rs:55`–`:62`: descriptor contains manifest/logical hash; `:117` decodes against retained session | Supported target provider/logical descriptors with exact versions/contracts/runtime rebinding. Explicitly refuse unsupported native/Delta codec arms; no physical-plan persistence requirement. |
| Old terminal fixture and ancestry route | `xtask/src/engineering_inspection.rs:270`: traverses `Snapshot::parents`; `:288` only invokes pending simulator targets | Fresh source-to-solve/publication/cold-open fixtures. Remove old golden-store and ancestry obligations, retaining independent domain assertions that still apply. |

**Already useful target code should stay.** The native fixed-point loop is a
specialized physical algorithm with actual inputs (`crates/pse-rules/src/strata/mod.rs:108`).
It is not a legacy engine merely because its inner algorithm is Rust. Likewise,
prepared sparse layouts, parser implementation and Ipopt callbacks belong inside
contracted native operations. A `CanonicalGraph` used transiently by an actual
numerical algorithm is distinct from retaining predecessor graphs as model authority.
No blanket rule to turn every algorithm into SQL, UDFs or persistent Delta rows is needed.

**The solver integration gap is concrete.** Product session construction installs
`pse_compiler::query_planner()` (`crates/pse-runtime/src/session_factory.rs:21`), whose
extension list currently contains `RuleExtensionPlanner`
(`crates/pse-compiler/src/lib.rs:40`). Solver tests explicitly install
`NativeSolverPlanner` (`crates/pse-backend-native/tests/native_solve.rs:48`). The
operator is real; the product runtime does not yet install it. Its effect admission
and prepare-once numerical program are implemented
(`crates/pse-backend-native/src/native/execution.rs:57`, `:123`).

### Provider-level completion, beyond implementing traits

| Level | Useful implementation | Remaining standardization |
|---|---|---|
| Runtime/session | Shared pools, actual state, composed Delta/native planners | Preserve configuration; install solver/backends; qualify foreign work and all command paths |
| Catalog list/catalog/schema | Native memory namespaces and exact publication bindings | Make them the only product opening path; complete discovery/absence/async/metadata contracts |
| Table/source/view | Exact Delta scans, generated layouts, validating writable adapter and owned captures | Complete domain admission, view/no-scan policy, pushdown/constraint proof and mutation coverage |
| Functions/expressions | Built-ins, native parser/support functions, compiled numerical expressions | Replace closed relational algebra; finish domain math/kernel typing/derivatives and ordinary extension proof |
| Analyzer/optimizer/planner | Common preparation and new extensions with real children | Remove root callbacks; native effects currently include concrete downcasts in `session/policy.rs:249`–`:264`; prove new extension coverage without another closed roster |
| Delta transaction/lifecycle | Validating builders, exact versions, conditional root and control reconciliation | Full write-graph retry, schema-change coverage, CDF, maintenance and live-reader retention |
| Streams/inspection/serialization | Owned native outputs and bounded failure/cancellation tests | Same durable solved state in fresh Rust/Python processes; target reconstruction and complete resource accounting |

## 5. Representative journeys

| Journey | What currently works | Where the target journey stops | Required observable completion |
|---|---|---|---|
| Author source → publish → reopen | `tests/engine/tests/unified_sources.rs:67` writes parsed source facts to Delta and reconstructs a fresh context at `:134` | Same OS process; not normalized heater/mixer cases, complete process compilation or solve | Fresh source fixtures through target model/case/run and separate Rust/Python processes |
| Source → process mathematics | Existing P10 test executes domain compilation with native inference | Uses old driver/store; successful P10 is not target M2 or M3 | All four heater/mixer-state combinations with independent topology/property/math/support assertions using only target providers |
| Numerical model → Ipopt | Actual nonlinear solve, derivatives, objective/duals, failures and cancellation pass | Hand-constructed expressions; no UD06 case builder, product planner registration or durable run | Source-derived connected flowsheet, initialization, solve and exact Delta result publication |
| Ordinary method/invariant/provider extension | Native registration and generated contracts are foundations | Closed rule/type/trace and effect rosters still create multiple editing sites; complete target extension journey unproved | One semantic declaration plus genuinely new implementation/tests; same discovery/admission/lineage/durability |
| Edit topology/policy → affected recomputation | Native change queries and old memo machinery exist | No target CDF/dependency/reuse/retention lifecycle | Delta edit and exact dependency queries, conservative recomputation where needed, independently justified reuse |
| Interrupted publication / abandoned solve | Root reconciliation and C-callback lifetime tests exist | Composed child-write retries, cold recovery and publication/reader cleanup races unqualified | Typed committed/unpublished/unresolved outcomes; no repeated ungoverned effect; live versions remain readable |

## 6. Acceptance gates

| Gate | Verdict | Evidence and limit | Required action |
|---|---|---|---|
| G1 — Authority | **Fail for the target cut** | Old compiler/store/Python authority remains beside Delta publication, without one product revision/root protocol (§2/§4). This is a structural failure of the required single lifecycle, not a claim that a corruption incident was observed. | Delete predecessor writer/read authority and make all product callers select target publications. |
| G2 — Semantic fidelity | **Unresolved** | Durable layout and scalar numerical tests are bounded; full Slice A, indexed/guard/kernel and backend round trips are missing. | Complete S02–S06/S09 scientific and representation oracles. |
| G3 — Validity | **Unresolved** | Candidate local/PK/FK checks exist; process completeness and all Rust/SQL/Python write/execute paths are not qualified. | Establish exact product invariant boundaries and adversarial bypass tests. |
| G4 — Hidden behavior | **Unresolved** | New operators declare useful behavior; raw read helper, root callbacks, config replacement and extension-effect completeness remain. No full cross-interface effect attack was executed in this audit. | Common preparation plus actual configuration/effect/EXPLAIN tests for every supported operation class. |
| G5 — Consistency/recovery | **Unresolved** | Conditional root/control retries do not prove composed writes, durable solve outcomes or reader/cleanup coordination. | V06–V08/V10 across every effect boundary and cold recovery. |
| G6 — Transformation/reuse | **Unresolved** | Owned lineage/native inference improved; old artifact memo and unimplemented target reuse/CDF remain. | Native semantic dependencies, optimized/unoptimized meaning checks and clean target recomputation oracle. |
| G7 — Capability claims | **Unresolved for full scope** | Real native solver replaces the former stub claim; feature-off execution refuses explicitly. Structural/initialization/Pyomo/NL and terminal simulator routes remain absent. Current partial status is truthful. | Implement required routes and complete executable acceptance; do not relabel focused tests as simulator completion. |

## 7. Principle findings

| Finding | Principle IDs / verdict | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| C01 — The old lifecycle remains product authority | DM-02, DM-14, DM-23 **Violated against the target** | Driver stage publication and memo; old Python opening; separate Delta root (§2/§4) | A Delta source publication cannot be the sole input to the existing compiler/Python lifecycle; target data still requires a second authority or unreachable functionality. | One source-to-model/case/run publication path; delete the old caller closure in the same cut. | Fresh target-only store completes product journey; source/call-graph deletion audit; no old object readers/writers. |
| C02 — Common native policy and composition are incomplete | DM-04, DM-22, DM-28, DM-44 **Unresolved**; DM-31 **Unresolved** for full configuration dependencies | `publication.rs:107`; `factory.rs:98`; `preparation.rs:332`; concrete effect downcasts | A caller can take a native read route without PSE preparation; caller config is replaced; native children/effects are not uniformly exposed. | Restrict raw helpers; preserve native config with explicit overrides; ordinary extension planning and complete effect admission. | Same denied operation through Rust/SQL/Python/view/no-scan/nested plans; sentinel settings/rules/UDFs survive. |
| C03 — Candidate checks do not establish a complete process publication | DM-07, DM-09, DM-22 **Unresolved** | `delta/admission.rs:38` checks supplied members, not full model/case/run obligations | A locally valid member vector has not proved it contains a usable complete product state. | Generated publication-kind completeness/domain checks over exact selected members. | Missing demands/equations/case fields/required results rejected before visible root; valid cases accepted. |
| C04 — Root retry evidence does not cover the write graph | DM-14, DM-29, DM-30, DM-35 **Unresolved** | `publication_plan.rs:48`, `:71`, `:79`; write children precede root reconciliation | Retrying after a child commit may repeat candidate work or select different versions; an attempt metadata string does not settle prior effects. | Durable attempt/input binding and member reconciliation; exact unchanged-member/slice composition. | Inject failure after every child/root step; same retry converges; changed inputs refuse; concurrent first-create/stale-parent tests. |
| C05 — Native lowering still requires a second relational language | DM-56, DM-57 **Violated against the target**; DM-24, DM-46 **Unresolved** for full replacement | `rule.rs:346`; `rule_validation.rs:176`; `plan/lower.rs:27`; `plan/trace.rs:100` | Adding a native relational capability may require new shape/type/lowering/trace interpretations; the architecture remains constrained by its own operator roster. | Use native plans/expressions directly; preserve only domain truth/support/absence/termination contracts. | Ordinary extension without new relational dispatch; multiplicity, unknowns, support removal and optimized-plan oracles. |
| C06 — Real solver is not yet a source-derived simulator | DM-22, DM-43, DM-44 **Unresolved** | Runtime planner lacks solver; `pse-structural`/`pse-plans` and Pyomo/NL `src/lib.rs:9` remain stubs | Authored cases cannot use the new solver through the product runtime; full supported backend outcomes are unavailable. | UD06 → product UD07 → UD08 using one problem contract and Delta outcomes. | Complete source-based cases, independent physics/derivatives and actual native/Pyomo/NL consumption. |
| C07 — Reuse and maintenance are still outside the target lifecycle | DM-31, DM-32, DM-35 **Unresolved** | `driver.rs:139`, `:155`; no completed target CDF/reader-pin/retention route identified | Old artifact reuse survives; target edits and cleanup have no qualified semantic reuse/live-reader contract. | Native dependency/diff queries plus Delta CDF/maintenance; delete stage hints/memo/context restoration. | Data/policy/function/absence changes; update pre/postimages; compaction neutrality; data/log cleanup races. |
| C08 — Python/reconstruction still require predecessor identities | DM-41, DM-42, DM-48 **Unresolved** | `inspection/handles.rs:105`, `:249`; `session/plan_codec.rs:58` | Target Delta model/run cannot yet be cold-opened and reconstructed through the advertised old handles. | Target handles/descriptors and shared native streams; delete manifest/hash identity requirement. | Separate OS-process Rust/Python reads, retained-stream/drop/cancel behavior, unknown-codec refusal. |
| C09 — Resource and completion evidence is narrower than required | DM-39, DM-53, DM-54 **Unresolved**; DM-59 **Satisfied only by explicit partial claims** | Foreign allowance at `crates/pse-backend-native/src/driver/workspace.rs:49`; absent terminal targets at `xtask/src/engineering_inspection.rs:302`, `:321` | An admitted allowance is not a measured/enforced C allocation bound; focused tests cannot establish product memory or scientific acceptance. | Measure actual foreign/retained resources, complete terminal fixtures, rerun current-source final gates. | Small/scaled target journeys with peaks, retained bytes, cancellation and file/log growth; zero-failure final receipts. |

**Applicability:** all twelve principle groups bear on this cross-cutting scope,
through authority, typed domain meaning, revisioning, composition, compilation,
effects, dependencies, cost, interfaces, lineage, verification and architectural
duplication. The findings cite the specific obligations examined, not all sixty
principles indiscriminately. DM-51 does not impose a historical-data migration:
existing development objects are explicitly disposable. Future target contract
versions still need truthful recognition/refusal. Remote/distributed deployment and
complete later-phase physics libraries are outside this completion scope.

### Traceability to the original review's findings

| Original finding | Current disposition | Remaining scope / this assessment |
|---|---|---|
| F01 publication | Partial | Exact Delta root implemented; product cut, completeness and lifecycle recovery remain — C01/C03/C04 |
| F02 representation | Partial | Generated durable forms exist; full math/backend/Python value/meaning round trips remain — C06/C08 |
| F03 write validation | Partial | Validating writes and persisted CHECK hooks exist; every mutation/schema route and process obligations remain — C02/C03 |
| F04 retry/outcome | Partial | Control reconciliation and solver terminal failures exist; composed writes and durable run publication remain — C04/C06 |
| F05 native composition | Partial | Delta/fixed-point/solver extensions have real children; old root envelope/controller/algebra remain — C01/C02/C05 |
| F06 common policy | Partial | Common publication-session binding exists; raw helper, config and full effect coverage remain — C02 |
| F07 artifact lifecycle | Open replacement | Old store/snapshots/stage/context/sidecar consumers remain reachable — C01/C07/C08 |
| F08 mathematics/numerics | Partial | Prepared scalar evaluation and real Ipopt work; full process/problem/kernel/backend scope remains — C06 |
| F09 lineage/dependencies | Partial | Owned support and exact selected facts exist; source-to-result and absence-aware target reuse remain — C05/C07 |
| F10 codecs | Open target route | Diagnostic codec still binds old manifests; target descriptor/replan support not closed — C08 |
| F11 resources | Partial | Shared resources/ownership/cancellation tests exist; foreign allocation enforcement/measurement and full boundary costs remain — C09 |
| F12 CDF/retention | Open | No completed native semantic CDF/live-reader/maintenance lifecycle — C07 |
| F13 exact dependencies | Partial / build demonstrated in focused scope | Actual combined dependency builds exist now; fresh whole-workspace/feature/Python closure remains — C09 |
| F14 obsolete constraints | Partial | Target decision and repository `AGENTS.md` now name Plan 07; old runtime/test obligations still require removal — C01/C09 |

No original finding is closed for its full product scope by a focused component test.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication / locality | Risk | Cost and performance evidence | Decision |
|---|---|---|---|---|
| Continue adding native wrappers around the current compiler/store | Two lifecycles and closed rule algebra remain; every integration retains old contracts | The first complete simulator would still depend on predecessor objects | Existing P10 timing includes old stage persistence; no target performance proof | Reject: does not satisfy hard-pivot deletion |
| Complete every provider feature before a product path | Could eventually unify, but adds broad abstractions before actual consumers | More partial components without process-level validity evidence | No measurement justifies this ordering | Reject as execution order; retain genuinely required provider contracts |
| Complete one target-only source→problem→solve→Delta→cold-open path, then expand the same path | One authority; native operators and typed Delta relations; conservative full recomputation until reuse is qualified | Narrow cases must not be mistaken for full Slice A acceptance | Smaller integration surface; measure before finer optimizations; full required scope remains | **Selected practical route to the same target** |

No new generic scheduler, semantic execution DSL, publication service or parallel
acceptance framework is justified. Use existing native plans, generated relations,
Delta operations and normal test/xtask machinery. Specialized matching, sparse
evaluation, parsing and FFI remain ordinary implementation code behind native contracts.

## 9. Verification and cost evidence

### Reviewed execution receipts

All counts below use **baseline 0**. These tests ran during implementation before
this documentation audit; they were not rerun for this review.

| Claim | Evidence / exact command | Mode and result | Limit |
|---|---|---|---|
| Actual Ipopt execution and boundaries | **Tested:** `just native-solver-test --no-fail-fast` | default; `ipopt` + `pse-relations/force-validate`; **12 passed, 0 failed, 0 skipped**, 2.100 s; run `7cb2ed4d-6c74-423f-a83e-7977ff4e32dc` | Constructed numerical inputs; not source-derived problem, full Slice A or durable run |
| Common session/cancellation regression | **Tested:** `just test-package pse-catalog --lib --no-fail-fast` | default/force-validate; **113 passed, 0 failed, 0 skipped**, 5.668 s; run `ad64da8c-974c-44df-b21c-3b2a47e32c89` | Includes old-store tests; does not prove deletion or all integration paths |
| Missing solver capability refuses | **Tested:** `just test-package pse-backend-native --no-fail-fast` | default/force-validate, Ipopt off; **2 passed, 0 failed, 0 skipped**, 0.010 s; run `7d5480ed-8f61-4077-80cb-d40c15969353` | Predates the final linked cancellation refinement |
| Source Delta round trip and existing compiler consumer | **Tested:** `just test-package pse-tests-engine --test unified_sources --test native_template_graph --profile ci --no-fail-fast` | CI/force-validate; **2 passed, 0 failed, 0 skipped**, 261.120 s; run `66282916-fb6f-4a20-81ce-b98c25674f38` | P10 case uses predecessor compiler/store; fresh context is not fresh OS process; default 120 s limit previously timed out |
| Generator/governance checkpoint | **Tested:** `just governance` | default/force-validate; **59 passed, 0 failed, 0 skipped**, 3.012 s; run `95ef2607-1c64-476a-a8c8-4a6549aea6a4` | Predates newest backend dependency/version-constant changes; not current final closure |

The solver receipt used the target runner in digest-pinned image
`ghcr.io/paul-heyse/pse-solvers:dev-e84fdce84e2f@sha256:39f6928ddcd3855ee097ac79244f87e3ebd4c9a2af371c44952e035e1ea45eb3`
(Ipopt 3.14.20, MUMPS 5.9.1, pinned solver distribution). Tests exercise a nonlinear
feasibility case, objective/dual oracle, nonlinear mixer/heater balance example,
invalid/infeasible outcomes, callback panic, cancellation, abandoned streams,
allocation-admission refusal and policy refusal. The mixer/heater example is manually
constructed numerical mathematics; it is not a source-derived engineering fixture.

**Open health evidence:** the current environment is outdated and the editable
extension was built from a stale lock. The earlier 65-finding compiler Clippy failure,
388-missing-fixture conformance failure and `proc-macro-error2 2.0.1` future-compatibility
warning have no final clearing receipt. Focused native/catalog Clippy passed before
the last cancellation change; it does not certify the final whole tree. Earlier
Python successes cannot qualify the current native source. Do not rebuild obsolete
fixture forms simply to clear their old assertions: replace them with required target
coverage and retain the zero-failure requirement for the completed tree.

### Original verification obligations

| Check | Current evidence | What must still pass |
|---|---|---|
| V01 dependency/product route | Combined focused builds exist | Current full feature graph, product runtime and backend paths, family and generated checks |
| V02 native composition | New Delta/fixed-point/solver child paths tested in parts | No root callback dispatch; actual sentinel native settings/rules/UDFs; all nested effects |
| V03 validation | Declared values/CHECK/PK/FK tests in parts | Every Rust/SQL/Python mutation and schema path; model/case/run completeness |
| V04 durable meaning | Durable layout and exact source round trips | Full values, metadata, nested/reference/quantity meaning and cold process boundaries |
| V05 optimization/support | Owned facts and finite inference tests | All target relational rewrites, multiplicity/unknown/support removal, mathematical guards |
| V06 publication | Exact root, first-create/conflict and selected opening tests | Failure after every composed child/root step, full product vector, cold OS-process opening |
| V07 retry | Complete control-record retry/reconciliation | Member-write retry/reconciliation, lost acknowledgments and changed-input identity across the graph |
| V08 resources/effects | Native ownership, cancellation, C panic and allowance-admission tests | Actual foreign envelope, Python abandonment, durable effects after cancellation, complete native policy timing |
| V09 engineering semantics | Scalar derivative/Ipopt examples and old P10 assertions | Source heater/mixer × FTPx/FcTP; declared property families; case/structure/scaling/init/indexed/backend coverage |
| V10 change/retention | No complete target receipt | Data/policy/absence/function dependencies, CDF update pairs, qualified reuse, data/log and reader-pin races |
| V11 reconstruction | Old diagnostic codec checks are not target closure | Supported target descriptor round trips, exact rebinding and safe unknown/unsupported refusal |
| V12 hard deletion | Many live predecessor callers identified | Removal of their runtime modules, schemas, callers, fixtures and stale operating policy |
| V13 terminal simulator/cost | Terminal recipe dispatch exists | `unified_simulator` and cold Python targets, whole source-to-run/backend journey, measured phase/peak/retained/file/log costs |

**Cost is unmeasured for the target simulator.** `Workspace` reserves an explicit
foreign allowance along with known numerical storage; this is an admission claim,
not instrumentation or a hard cap on Ipopt/MUMPS allocations. The allocation lease
also remains attached to output ownership. Measure release/retained behavior and
define enforceable limits before claiming the full resource contract. The old P10
timing is diagnostic evidence about that old route, not a target benchmark.

### Documentation assessment checks

**Tested — documentation only, baseline 0:** scoped `.venv/bin/typos` over this
review, its evidence JSON, Plan 07 and `STATUS.md` reports **0 findings**. A targeted
Python check validates front matter, all eleven review sections, all package/F/V
rows and 15 local links with **0 errors**. `git diff --check` on the two updated
tracked documents reports **0 findings**. These checks do not exercise runtime code.

`just docs` builds the HTML book with **0 errors and 2 warnings, baseline 0**: an
unclosed `<new-output-directory>` tag in proposed ADR-0068 and the large search
index. Neither warning originates in the new review. They remain open; successful
book generation is not a zero-warning documentation receipt.

## 10. Exceptions and unresolved decisions

No compatibility, legacy-data retention, duplicate authority or restricted native
function allowlist exception is granted. The accountable implementation stream is
Codex under the user's authorized Plan 07 scope. Gaps remain open until their named
checks pass; recording them does not authorize an alternative architecture.

Resolve within existing packages: complete publication-kind obligations; durable
attempt/input reconciliation across member writes; exact reuse/slice composition;
native configuration override rules; common effect admission for new extensions;
foreign-memory enforcement/measurement; target descriptor support; and reader-pin
coordination for data files **and** logs. These are concrete implementation decisions,
not reasons to preserve old runtime objects or to delay the target for a new platform.

## 11. Decision and prioritized implementation changes

**Revise the implementation, retain the target.** The next completion claim must be
about a target-only functional path and its deleted predecessor closure. More isolated
green components cannot close M1 or make the existing compiler a Delta compiler.

| Priority / dependency | Functional cut | Principles | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 — close authority and common boundary | UD01–UD04: one exact Delta source/model opening and native command path; complete policy/config binding and composed publication recovery; cut old store/compiler/Python opening callers together | DM-02/14/22/28/30 | Target source→publication→cold Rust/Python query; invalid/missing members and retry failures; no old refs/manifests/stage store needed | Tests start from target-only directories; caller/deletion audit |
| 1a — prerequisite integration inside that cut | Move reusable compiler transforms to native plans over selected providers; remove the old driver/stage-publication requirement as callers move | DM-23/25/57 | Required process facts produced without a `Snapshot` input or restored stage sidecars | Target fixture generator and independent fact assertions |
| 2 — complete native process construction | UD04–UD06: native normalization/inference/math; delete closed relational algebra; construct cases, structure, scaling and initialization | DM-07/09/24/46/56 | Heater/mixer × both state bases, exact Slice A coverage, indexed/guard/discretization and malformed-case oracles | Native extension and optimized/unoptimized semantic cases |
| 3 — connect actual solving and results | UD07–UD08: install product planners; shared problem → Ipopt/Pyomo/NL/SOL; publish typed run outcomes and reopen them | DM-22/29/37/42/43 | Source-derived nonlinear connected case; independent balances/derivatives; failed/cancelled/durable outcomes; actual alternate routes | Pinned solver/container and backend integration suites |
| 4 — complete change and operating lifecycle | UD09–UD10: exact semantic dependencies, conservative recomputation/qualified reuse, CDF, pins/maintenance, target inspection/reconstruction | DM-31/32/35/48 | Meaningful edit and clean target recomputation agree; reader/cleanup races; cold Python owned streams | Version-range, absence/policy, codec and retention tests |
| 5 — certify the whole scope | UD11–UD12: full terminal acceptance, extension proof, costs, current-source quality and final independent gates | DM-39/53/54/59/60 | Fresh S01–S11/V01–V13 receipts, zero failures, no reachable legacy path | Normal tests plus one terminal recipe; deletion audit repeated after all consumers land |

The priority rows are dependency-ordered work within one hard pivot, not staged
releases. Deletions belong to their owning functional cuts; UD12 verifies absence
rather than becoming a holding area for postponed predecessor removal.
