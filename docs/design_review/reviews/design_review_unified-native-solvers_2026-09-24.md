---
title: Unified native solver implementation review
status: accept-scoped
date: 2026-09-24
scope: Plan 14 M10–M16 contracts and targeted package acceptance
evidence: Tested — 105 selected native units plus public boundary controls; full native convergence and qualification remain M22
---

# Unified native solver implementation review

## 1. Decision and scope

**Accept scoped:** the M10–M16 implementation supplies one Rust solver lifecycle
with class-specific library adapters, qualified library transformations and a public
model/result workflow. This is acceptance of the implemented and
unit-tested contract boundary, not scientific or end-to-end simulator qualification.
The [execution packet](../../plans/14-m10-m14-execution.md) owns exact scope,
profiles and verification; the [inventory](../../plans/14-execution-inventory.md)
owns remaining work. ADR-0083 remains proposed pending formal decision reconciliation.

**Method and coverage:** inspected compiler queries, class routing, native upload and
callback boundaries, original-space reports, finite worker/session lifetimes,
conditional initialization, physical graph binding and tear witnesses. Exercised
95 selected force-validation units in the pinned native container and compiled the
full native and default runtime profiles. Negative controls attack callback partial
writes/panics, invalid bounds/domains/classes, sparse identity ambiguity, stale inputs,
conditional demands, map roles, ownership and graph policy. Native convergence,
native teardown under concurrent solves, public Python workflows, publication,
scientific comparisons, performance, other targets and feature combinations were not
executed; they remain M16/M20/M22. No speedup or hard RSS limit is established.

The preceding 95-test receipt describes M10–M14. The M15–M16 update additionally
inspected generated declaration admission, shared TNLP wrappers, original-space
observation, immutable revision preparation, terminal result sharing, PyO3 blocking/
async cancellation, Arrow buffer retention and pure publication preparation. It ran
105 selected native units and public Python boundary units, with zero failures;
exact commands and supported API scope are in the
[M15–M16 packet](../../plans/14-m15-m16-execution.md). Storage effects, long native
cancellation, cross-backend convergence and the full process/Python/provider journey
remain unexecuted. Whole-plan Clippy is not clean and remains a qualification item.

The baseline was the M00–M10 library-owned foundation with native problem views but
no replacement solver lifecycle. The new observable capability is an admitted Rust
path from immutable mathematical preparation to a native attempt and an attributable
result, including initialization and tear strategies. No historical solver fallback
is retained.

## 2. Authority and lifecycle map

### Implemented M15–M16 extension

**Accept scoped / Implemented:** the [M15–M16 packet](../../plans/14-m15-m16-execution.md)
extends these boundaries with shared library presolve, original-space observations,
typed authoring and public jobs. Inspection of pounce-presolve 0.12.0 establishes
the expression-provider requirement, affine-offset normalization seam, fixed
tolerance limits, wrapper recovery order and tape-blind fingerprint. Independent
original evaluation is mandatory because failed native finalization can provide
placeholder values. Symbolica owns row proofs; Salsa owns pure dependencies;
native wrappers and Python cancellation remain completion-owned effects. Exact
Delta settlement is reused. The targeted units exercise these contracts, including
failed original observation, all-column stand-down, explicit native options,
maximization/scaling, immutable revision edits and final Arrow reader ownership.
This extends contract acceptance, not native convergence or storage fault assurance.

| Fact | Authority and identity | Owner / update boundary | Derived form |
|---|---|---|---|
| Mathematical definition and physical binding | Typed source, physical registry, CaseStructure and BodySpec keys | Compiler inputs, atomic workspace update | Immutable CasePlan, coefficients, derivative artifacts |
| Class eligibility | ProblemFacts plus exact current convexity evidence | Pure compiler products; runtime admission | Deterministic Route or explicit refusal |
| Execution policy | SolverProfile / Controls / physical Tolerances | Prepared request; finite sequence boundary | Protected native settings, native thread/limit policy |
| Graph and initialization meaning | Complete flow declarations, connection occurrences, decision groups, equality incidence | Salsa-tracked inputs/projections and BTF schedule | petgraph projection, MILP columns, conditional programs |
| Native state | Selected library object, allocation/data compatibility, owned warm seed | Admitted owner thread only; no Salsa mutation | Native factors, basis, iterates and statistics |
| Result | NativeTermination, optional candidate/certificate, independently recomputed Quality | Owned envelope with retained allocation lease | Generated physical Arrow rows; explicit exact-version publication |

Coefficient/cone proof validation also runs on admitted math workers; it does not
perform Symbolica work on the async caller. Tear selection retains one native report
next to its independently checked graph witness.

Native convergence, factorization and heuristic graph algorithms are deliberately
opaque library behavior. The project declares their eligibility, budgets, data maps
and outcome interpretation. It does not encode these algorithms in a second IR.

## 3. Semantic contracts and invariants

| Contract | Enforcement and code evidence | Failure / evidence |
|---|---|---|
| No implicit relaxation/reformulation | `crates/pse-backend-native/src/routing.rs:19`, `select`; `crates/pse-runtime/src/math/solves.rs:186`, `admit_profile` | Typed ineligible/unavailable result; class-refusal units |
| Sparse identity preserves each column | `crates/pse-math/src/sparse.rs:100`, `pattern_key` frames dimensions and column lengths | Two equal-length flattened supports differ in the regression control |
| Callback outputs and terminal cause remain faithful | `crates/pse-backend-native/src/callback.rs:81`, `catch_unwind`; adapter temporary buffers | Recoverable trial stays nonterminal; panic/contract failure clears assurance; direct callback units |
| Original-space quality is separate from native success | `crates/pse-backend-native/src/solve.rs:535`, SolveReport; `crates/pse-backend-native/src/kinsol.rs:730`, fallible Quality handling | Validation error preserves raw native outcome; per-coordinate physical tolerances retained |
| Native reuse must respect library restrictions | `crates/pse-backend-native/src/conic.rs:202`, `is_data_update_allowed`; runtime compatibility and native pattern checks | RequireReuse refuses incompatible state; no external Clarabel iterate-start claim |
| Partial blocks cannot commit trials | `crates/pse-runtime/src/math/initialization.rs:430`, `commit_block` | Native success plus original quality and complete finite coordinates required; atomic-commit unit |
| Complete flow semantics survive optimization | `crates/pse-backend-native/src/tears.rs:26`, order-variable MILP; CausalMap source/output and destination/input admission | Independent DAG witness; exhaustive tiny graph, group, self-loop, physical-conversion controls |

Absent metrics/duals/certificates use explicit optional values. Limit exits do not
become optimality claims. Semi-variable zero branches remain admissible without
filling their positive-domain gap. Infeasibility, unboundedness, local stationarity,
original feasibility and heuristic tear assurance are distinct.

## 4. Derivation and execution design

| Stage / RCA contract | Representation and dependencies | Equality, bounds, effects and retention |
|---|---|---|
| Semantic/class preparation | Salsa `problem_facts` derives from CasePlan and requested coefficient snapshot (`crates/pse-compiler/src/workspace.rs:575`) | Explicit values/structure, not a caller's guessed class; membership and absence tracked; no solve inside a query |
| Conditional initialization | `initialization_blocks` (`crates/pse-compiler/src/workspace.rs:634`) reads complete equality selection, schedule and physical registry | Unselected variables fixed; unrelated rows/objective removed; artifact requests retain exact demand/profile; no objective callback during root initialization |
| Flow/tear preparation | Complete directed multigraph, isolates, occurrence IDs, decision costs/policies, physical port conversion | Stable semantic maps; no endpoint collapse or implicit weighting; linear-size MILP and independent petgraph acyclicity witness |
| Native execution | Class-native representation, immutable artifacts, explicit values, settings and starts | Finite steps, native limits and cancellation; all permits acquired once; mutable native models stay on the worker |
| Source/revision admission | Generated `authored.computation_models`; `workflow/model.rs` validates fields, IDs, bindings and physical context before freeze | Atomic immutable revisions; no default fixture catalog; document and typed rows share admission. Source bytes and retained batches have pool owners. |
| Presolve facts and transformations | `crates/pse-math/src/presolve.rs:84`, `presolve_facts`; `crates/pse-backend-native/src/presolve/pipeline.rs:91`, `Pipeline::new` | Pure bounded source projection is Salsa-owned; native wrappers are attempt-owned. Guards/opaque expressions remain explicit; library iteration/recovery is not memoized. Maps, values, scales and tapes enter identity. |
| Original observation and public output | `Pipeline::finish` at `crates/pse-backend-native/src/presolve/pipeline.rs:411`; `crates/pse-runtime/src/workflow/results.rs:23`, `tables` | Fresh source evaluation after native recovery; errors preserve native status. One immutable encoding, finite report bounds and final Arrow buffer leases. No hard native RSS claim. |
| Public job and publication | `crates/pse-py/src/workflow.rs:355`, `wait_async`; `crates/pse-runtime/src/workflow/publication.rs:64`, `prepare_publication` | One process executor and native join owner. Repeated wait reads terminal state. Publication preparation has no writes; a one-use commit invokes existing exact member/control settlement, never a solver. |
| Native teardown | HiGHS read/exclusive lifecycle gate (`crates/pse-backend-native/src/highs.rs:84`); scoped Rayon (`crates/pse-backend-native/src/pounce.rs:375`) | Blocking scheduler reset/pool join precedes admission release; completion-owned thread join includes TLS cleanup |
| Result construction | Native code/status, original coordinates/quality, available diagnostics and source IDs | `run_sequence_inner` (`crates/pse-runtime/src/math/solves.rs:640`) attaches result leases, stops dependent work after failure and drops poisoned state; no durable writes |

Graph connectivity, equation incidence and block dependencies are distinct
projections. Numeric recycle is a declared map passed to KINSOL; recursion has the
library's convergence and stopping semantics, never Salsa query recursion or a custom
loop. Report allowance includes explicit option strings and bounded event extents;
foreign/native temporary storage remains a conservative admitted allowance. Exact
floating-point equivalence across solver algorithms is not claimed. Source physical
quality and explicit tolerances provide the shared acceptance vocabulary.

## 5. Representative journeys

| Journey | Mechanism and consequence |
|---|---|
| Ordinary extension | A new native option uses the selected library's validation/readback; an option changing portable semantics must instead extend the typed profile and conformance controls. No new parser or solver registry is required. |
| Meaningful change | A fixed/parameter edit invalidates consumed coefficient assumptions while compatible body artifacts remain reusable. A sparse-pattern/body/profile change alters layout compatibility; a numeric update uses native update APIs only when eligible. |
| Alternate representation | HiGHS consumes sparse coefficients with domain/sense/constant; Clarabel consumes explicit cones with certified Q and bound-row maps. Neither synthesizes an unsupported representation from an opaque NLP. |
| Failure/interruption | Recoverable callback trials can continue; terminal errors stop native use. Limited native iterates remain candidates, not successes. Failed conditional blocks leave committed coordinates untouched. Dropped callers request cancellation while the completion owner retains resources through teardown. |

## 6. Acceptance gates

| Gate | Verdict at the stated packet boundary | Evidence / excluded claim |
|---|---|---|
| G1 — Authority | Pass | Typed/compiler products own mathematics; libraries own native algorithms; no mutable native state in Salsa. Formal blueprint alignment remains an explicitly pending decision process. |
| G2 — Semantic fidelity | Pass | Direct weighted Hessian, source-space quality, exact domain admission, native status/certificate separation and graph occurrence controls. End-to-end scientific fidelity remains M22. |
| G3 — Validity | Pass | Checked native uploads/indices, finite controls, profile and convexity admission, transactional callbacks and explicit unsupported cases. |
| G4 — Hidden behavior | Pass | Solver choice is explicit/deterministic; no option-file influence or silent fallback. Native diagnostics are requested and scoped; initialization map calculations have an explicit unit contract. |
| G5 — Consistency and recovery | Pass | Finite sequence outcomes, successful-block commits, callback panic containment and completion-owned resources. Durable publication and native concurrency stress are outside this packet's assurance. |
| G6 — Transformation and reuse | Pass within qualified subset | Source proofs, native maps/scales and expression-sensitive identities govern the shared presolve pipeline; independent original observations qualify recovered values/duals. Unsupported passes refuse or report ineligibility. |
| G7 — Truthful capability claims | Pass | Available classes, native profiles, missing interfaces and test boundaries are recorded. Full solver/process acceptance is explicitly unclaimed. |

These verdicts do not close Plan 14's terminal gates. An aggregate product verdict
remains unresolved until M17–M22 complete and qualify the target consumers.

## 7. Principle findings

| Finding / disposition | Principle verdict | Concrete evidence or gap | Consequence and correction | Verification |
|---|---|---|---|---|
| F1 — Sparse column boundaries, corrected | DM-15/DM-32, RCA §3: satisfied | `pattern_key` includes each column's length instead of concatenating row numbers alone | Prevents a warm-layout collision between different variable supports | `sparse_layout_identity_frames_column_boundaries`, selected unit run |
| F2 — Extracted result ownership, corrected | DM-29/DM-35, RCA §8: satisfied for runtime envelopes | `SolveReport::with_owner` and ConstantReport retain the result lease through extraction/clone | Dropping the batch cannot prematurely release its owned report allowance | `constant_sequence_uses_shared_lifecycle_and_retains_result_allowance`; native envelope attachment inspected |
| F3 — Native outcome preservation, corrected | DM-08/DM-30/DM-46: satisfied | Postsolve quality errors and optional HiGHS diagnostics remain explicit fields | Diagnostic failure cannot erase native termination or fabricate a successful candidate | Quality/failure units; native terminal preservation paths inspected |
| F4 — Whole-process resource/performance claims remain open | DM-39/DM-54, RCA §8–§9: unresolved outside packet claim | Foreign allowance is not allocator interception; no convergence/teardown/RSS campaign run | M22 must exercise native concurrent teardown, cancellation latency and actual peak memory before declaring bounded operational performance | Q07–Q10/Q16 plus target measurements |
| F5 — Full process acceptance remains open | DM-44/DM-53/DM-59: unresolved outside packet claim | M16 implements the generated algebraic/public result workflow; advanced cone/flow/provider construction remains typed Rust, and no full installed process journey was exercised | Do not describe these package units as complete Python capability parity, a scientifically qualified simulator or numerical IDAES parity | M20 must reconcile the required Python/provider acceptance case with this explicit construction boundary; M22 exercises the chosen target |

**Applicability:** DM semantic authority/types, identity, transformation, execution,
dependency/ownership, native boundaries, lineage and adversarial verification apply
because five distinct numerical representations share one model/runtime. RCA §1–§9
apply to pure/native selection, dependency completeness, graph meaning, explicit
numeric iteration and resource ownership. Durable transaction details and temporal
Delta graph semantics are not changed here. No event-time graph, distributed solver
is asserted. M16 adds generated Python result contracts and reuses, rather than
reimplements, the durable transaction protocol. DM-03/DM-25/DM-38/DM-41 and
DM-56–DM-58 are satisfied by the narrow adapter composition below rather than a
universal solver IR.

## 8. Alternatives and architectural leverage

| Alternative | Correctness and extension cost | Performance evidence | Disposition |
|---|---|---|---|
| Legacy interpreter/Pyomo construction | Two mathematical authorities and Python/native rebuild boundaries; incompatible with the hard pivot | Historical receipts do not qualify the new target | Deleted; no fallback |
| Shared lifecycle with native class representations | One admission/report/quality contract; five mechanical native adapters; native algorithms and statistics retained | Targeted behavior only; no measured speedup | Selected |
| Simpler fresh native model per call | Fewer retained-state branches, same underlying oracle | No performance comparison yet | Still available as `ReusePolicy::Fresh`; insufficient alone for the approved finite repeated-solve requirement |
| Universal model/option abstraction or mandatory Oximo translation | Would erase specialized callbacks, cone structure, root strategy and native diagnostics, or recreate them in a second model | No evidence justifies that intermediate layer | Rejected for this scope |

The shared contracts have current consumers across all five adapters, initialization
and tears. Further abstractions need a concrete consumer. Mechanical sparse mapping,
source attribution, tolerance application and callback containment remain ordinary
Rust; iteration, factorization, optimization and graph algorithms remain library-owned.

## 9. Verification and measurement plan

| Claim | Evidence label and conditions | Result / next evidence |
|---|---|---|
| Selected adapter/compiler/runtime/public contracts | **Tested:** `just unit-native-contracts`; Rust 1.98.1, pinned native container, local license, full native features and force-validation | 105 passed, 0 failed, 58 excluded; baseline zero |
| Full native and default compile surfaces | **Interface-checked:** `just check-solver-contracts`; `just check-package pse-runtime`, all targets | Both pass; existing transitive proc-macro-error2 future-compatibility notice remains |
| Public Python declaration/job/result boundary | **Tested:** `just py-native-contracts`; CPython 3.14.7, native-solver/force-validation editable extension and pinned native paths | 25 passed, 0 failed; baseline zero. Full process/provider and installed publication journeys remain M22 |
| Artifact compatibility and generated boundary | **Tested:** `just unit-package pse-model 'test(artifact::durability_unit::)'`; explicit force-validation; **Interface-checked:** `just python-contracts-check`, actual native stub generation and `just family-check` | 1 unit passed, 0 failed, 32 excluded; baseline zero. No legacy restoration or migration route |
| Native lifetime design | **Implemented**, inspected RAII/gates plus **Tested** shared completion/TLS and retained-result units | Actual simultaneous solves, blocked factorization and teardown remain M22 |
| Numerical convergence and physical science | **Proposed:** Q07–Q10 and independent analytic/physical references | M22; cross-backend agreement alone is insufficient |
| End-to-end cost / peak RSS / repeat-solve benefit | **Proposed:** M20 workloads and M22 measurements | No measurements claimed |

## 10. Exceptions and unresolved decisions

| Item | Scope and disposition | Owner / trigger |
|---|---|---|
| Native transformations | M15 implements a qualified subset with original-space observation; unsupported tapes, narrow intervals, small coefficients and unsafe policies are refused or explicitly excluded. All-column elimination follows the pinned library stand-down | M22 numerical comparisons; widen only with a concrete qualified library contract |
| Feature/platform/convergence qualification | Default and full Linux native profiles compiled; independent combinations, real solver journeys and other targets not certified | M22 after M00–M21 closure |
| Native memory and cancellation | Admitted allowances and actual ownership; no universal hard RSS or immediate foreign-call preemption promise | M22 measurements before operational claims |
| Blueprint/ADR status | User-authorized hard pivot implemented; accepted blueprint is unchanged; ADR-0082–0084 remain proposed | Formal design/decision reconciliation, M19–M22 |

## 11. Decision and implementation changes

| Priority | Decision / action | Completion condition |
|---|---|---|
| Current | **Accept scoped** M10–M16 contract implementation | Targeted units and compilation above; no full simulator assurance inferred |
| Next | Execute M17–M18 with the same model, native lifecycle and physical result contracts | Native dynamics and fitting/sensitivity, with declared unsupported cases |
| Then | Execute M19–M21 and keep capability claims bounded | Final cleanup, current capability inventory and implementation/deletion closure |
| Terminal | Execute M22 once after implementation/deletion closure | Native convergence, independent science, lifecycle stress, Python, performance and whole-plan gates pass; formal decision reconciliation complete |
