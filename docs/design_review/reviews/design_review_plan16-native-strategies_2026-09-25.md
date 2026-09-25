---
title: Plan 16 P07–P09 native strategies and dynamics
date: 2026-09-25
status: complete
scope: implementation and approved execution contract
evidence: Tested — targeted contracts, complete scoped Rust/Python regressions and required static checks
---

# Plan 16 P07–P09 native strategies and dynamics

## 1. Scope, purpose and coverage

| | |
|---|---|
| Subject | [Execution packet](../../plans/16-p07-p09-execution.md), native routing/quality/starts/dynamics, runtime strategies/fitting, physical valve law and public Python bindings |
| Standard | Core 2.0, process-simulator 1.0, pse-arrow binding |
| Tier · purpose | Design · conformance to the approved target |
| Reviewer · date | Codex implementation self-review · 2026-09-25 |
| Decision | Accept scoped; slot 12 |

Examined: contextual route admission; original versus uploaded coefficients; native stop
versus numerical qualification; explicit input starts versus output seed availability;
immutable initialization overlays; selected tear graph and compiled causal maps; declared
time origin and prepared sample binding; library-owned event/reset sensitivities; IDAS
trial failure, consistent initialization and callback lifetime; declared valve smoothing;
candidate response derivatives versus qualified estimates. Targeted tests attack invalid
bounds/threads, unavailable routes, unacceptable optimality claims, seed/reset dependence,
failed continuation, nonzero-origin fitting and unsupported hybrid combinations.

This is not an independent review. Complete default/native Rust tests, Python tests and
public process journeys passed after the implementation phase. The new conserved
back-pressure journey and rebuilt Python strategy boundary are exercised. Complete
publication round trips for strategy reports, cache redesign, general sparse fitting,
performance campaigns and other-platform/release acceptance are outside P07–P09.

## 2. Authority and identity map

| Meaning | Authoritative type / owner | Identity and update | Derived view |
|---|---|---|---|
| Usable algebraic route | `routing::Requirements` over admitted `ProblemFacts`, controls and caller inventory | Selected structure, bound shape, derivatives and effective policy | Alternatives and preparation refusal |
| Numerical outcome | `SolveReport`, `quality::qualify`, completion assessment | One attempt and original evaluated candidate | Native stop, qualification, physical usability and diagnostics |
| Numerical start | `WarmStart`, `StartPolicy`, `StartReceipt` | Backend/layout/data compatibility plus origin run/attempt | Selected seed parts, transformations and submission receipt |
| Temporary stage | `InitializationProfile`, `StageAttempt`, `BlockAttempt` | Immutable original bindings, stage overlay and structural `BlockId` | Candidate overlay and separately committed unknowns |
| Recycle | Selected `PreparedFlow`, authored tear policy and explicit causal request | Revision, scalar port identities, graph witness and declared map bindings | Ordered compiled unit map and KINSOL history |
| Dynamic clock | Authored dynamic declaration and `workflow/time.rs` | Origin, start, unit and observation basis | One prepared integration sample index |
| Valve formulation | Registry `directional_valve_laws` and bound positive width parameter | Declaration and provider input participate in selected model identity | C2 kernel used by the same compiled model |
| Fit result | Shared fit oracle and `FitReport` | Candidate and response Jacobian at that candidate | Estimate qualification, rank and conditioning; no inferred uncertainty |

The explicit cone request is a typed analysis representation using Clarabel matrices and
cone variants. It does not reinterpret arbitrary process constraints as cones or create
another process compiler. Physical coordinate types still come from the quantity registry.

**Physical semantics:** dynamic coordinates are canonical time; observations declare
elapsed or model-clock coordinates and a compatible unit. Valve width has pressure
dimension, is positive and fixed, and selects the approximation explicitly. Original
quantity budgets and balance closure retain the P05 owner. Property packages, phase
selection and thermodynamic reference conventions remain the selected model's meaning.

## 3. Contracts and invariants

| Invariant | Enforcement | Failure behavior / evidence |
|---|---|---|
| A linked library is not automatically an exposed route | `Requirements.available` intersects linked capabilities with runtime inventory | Preparation refuses absent, bound-incompatible, derivative-incompatible or thread-incompatible routes; targeted routing tests |
| Native success alone cannot grant stationarity or optimality | `quality::qualify` requires original feasible quality and relevant KKT/gap evidence | Candidate remains feasible/unqualified when required evidence is missing; acceptable/gap qualification units |
| Coefficient upload cannot silently change the solved problem | Complete HiGHS readback plus fresh original oracle checks | Upload mismatch prevents optimality; defective-upload control |
| Allocation reuse does not select a numerical seed | Explicit `StartPolicy`; retained HiGHS state cleared; compatible seed applied separately | Missing/invalid explicit seeds fail; multiroot sequence test |
| Stage failure cannot mutate the original specification | Per-stage clone, separate committed unknowns, failed candidate retained | Failed continuation test; cancellation uses the same uncommitted path |
| A residual is not a declared fixed-point map | Explicit input/output port bindings and causal order | Unbound inputs, invalid witness or unsupported topology refuse preparation |
| Sampling has one time interpretation | Preparation converts coordinate and binds sample identity | Nonzero-origin analytic and fitted controls |
| Recoverable trials differ from terminal failures | `CallbackState` and IDAS positive/negative callback returns | Native recovery or terminal report; no bespoke time-step retry loop |
| Response derivatives are not estimator uncertainty | Fit rank/condition and estimate qualification stored separately | Rank-deficient or unconverged candidates cannot become qualified estimates |

Structural deficiency checks, original-domain obligations and physical validation remain
shared. Unsupported hybrid/layout combinations fail explicitly. Missing diagnostics stay
missing; the adapters do not manufacture unavailable iteration histories or derivatives.

## 4. Derivation and execution

| Stage | Mechanism | Reuse, effects and limits |
|---|---|---|
| Prepare | Shared facts/numerics, contextual selector, explicit cone/map admission | Immutable prepared ownership; derivative and thread requirements checked before solve submission |
| Seed | Validate backend/coordinates/parts, normalize selected values, apply native seed options | Independent of allocation policy; actual input provenance distinct from returned available start |
| Initialize/recycle | Structural blocks, authored tear decisions and causal compilation | Native KINSOL root/fixed-point/Anderson or constrained NLP; continuation transacts per stage |
| Solve/qualify | Native algorithm, native status translation, transport recovery, original checks | Native limits/cancellation retained; no universal success predicate replaces qualification |
| Integrate | Diffsol same-layout hybrid helpers or narrow IDAS residual adapter | Diffsol owns event-time/reset sensitivity; IDAS owns recovery, KLU, forward sensitivities, quadrature and consistent initialization |
| Fit | Same prepared model and time binding, native optimizer and valid response Jacobian | Candidate responses retained independently; rank/conditioning qualify the estimate, not confidence intervals |
| Expose | Owned Rust strategy reports, PyO3 lifetime/join, msgspec document envelope | Rust typed admission owns analysis meaning; complete durable result projection remains P10 |

The IDAS adapter's RAII session owns native context, vectors, matrix, linear solver and
integrator memory. Each C callback catches unwind and publishes buffers only after a
successful evaluation. The current residual uses fixed identity/zero mass rows; variable
mass, higher-index reduction and hybrid IDAS sensitivities are not supported claims.
Diffsol's numerical time-partial contribution is disclosed; it is not labelled analytic.

## 5. Journeys

| Journey | Observed or inspected result |
|---|---|
| Boxed square root request | KINSOL is ineligible; shared selector admits a constrained alternative or refuses before solve |
| Explicit cone request | Library geometry, exact Gram witness and physical coordinate policy produce a public route; Rust and Python targeted units passed |
| Repeated multiroot solve | NoPriorStart uses the declared point even with retained allocation; PreviousAccepted records the selected origin; targeted test passed |
| Failed continuation | Original bindings remain unchanged; failed overlay and partial attempts remain inspectable; targeted test passed |
| Selected recycle | Authored tear policy plus acyclic witness and explicit compiled causal map; public strategy convergence test passed |
| Smooth/recoverable residual dynamics | IDAS index-1 consistent state and sensitivities, recoverable trial and terminal failure controls passed |
| Evented parameter recovery | Nonzero origin, scheduled reset and state-triggered reset agree with independent piecewise analytic response; targeted fit test passed |
| Back-pressure limit | Declared C2 nonreversing valve derivative/join and conserved approach-to-back-pressure/reverse-closure tests passed |
| Native cancellation/resource limit | Owned join and partial/candidate reports inspected; actual failure/panic/limit/cancellation regression passed |
| Original optimality claim | Presolve-tightened bound multipliers and default QP regularization can fail original checks; candidate retained as feasible. Explicit presolve-off and smaller QP regularization controls satisfy the stronger qualification. |

## 6. Gates

| Gate | Verdict | Evidence and outstanding boundary |
|---|---|---|
| G1 Authority | Pass | One contextual selector, numerical owner, source model and explicit stage transaction |
| G2 Semantic fidelity | Pass | Typed stop/qualification, seed provenance, physical width and one time conversion |
| G3 Validity | Pass | Invalid routes, shapes, physical coordinates and unsupported dynamics rejected; complete public regression passed |
| G4 Hidden behavior | Pass | Start and reuse independent; no hidden valve fix or fallback; inspection reads retained reports |
| G5 Consistency and recovery | Pass in scoped ownership | Original/stage/candidate separation and native owned destruction; no new persistence claim |
| G6 Transformation and reuse | Pass in inspected contracts | Original coefficient validation, complete upload readback and seed compatibility; targeted controls passed |
| G7 Truthful capability claims | Pass | Complete scoped Rust/Python and required final checks passed; exclusions remain explicit |
| G8 Library leverage | Pass | Native algorithms own iteration/recovery/sensitivity/graph optimization; slot 8 |
| PS-G1 Physical model validity | Pass | Declared valve is unit-checked and C2-tested; conserved back-pressure and reverse-closure journey passed |
| PS-G2 Structural well-posedness | Pass | Shared incidence versus topology, exact/heuristic tear witness and explicit causal mapping |
| PS-G3 Numerical trustworthiness | Pass | Analytic controls, original KKT/gap downgrade cases, full scoped regression and rebuilt Python boundary passed |

## 7. Findings

| ID | Severity / finding | Principles · gates | Evidence and consequence | Correction / settling check |
|---|---|---|---|---|
| F01 — closed | Qualification gap | DP-22/23, PS-03/10/13 · G7, PS-G1, PS-G3 | Targeted tests alone could not qualify the final boundary or conserved back-pressure journey. | Final default/native/Python suites and affected static/generated leaves passed; slot 10 and execution Outcome |

No unresolved MUST defect remains in the examined supported scope. Errors repaired include
stale preparation/status assertions, Python transport typing, missing regenerated fixture
fields, an overly small fit fixture budget with a retained duplicate oracle, and an
uncontained outer C callback boundary. The native presolve/QP qualification limits found
by regression are explicit supported outcomes, not certificates inferred from status.
No unexecuted broader performance, persistence or numerical regime is promoted to a pass.

| Principles | Verdict | Mechanism / scope |
|---|---|---|
| DP-01/05/06/17 | Satisfied | Authored model, shared facts/policy, typed native adapters and completion retain distinct owners |
| DP-02/03/21 | Satisfied in inspected contracts | Typed outcomes, explicit support refusals, seed and transformation provenance |
| DP-04/09 | Satisfied | Revision/scalar identities, block identity, policy-bound preparation and seed compatibility |
| DP-07/12 | Satisfied | Topology, incidence and causal execution are different projections with explicit witnesses |
| DP-08/11/15 | Satisfied in targeted scope | Original-model checks, reversible coordinates and analytic hybrid controls |
| DP-10/18/19 | Satisfied | Prepared data and completed reports own lifetime; original source stays immutable |
| DP-13/14/16 | Satisfied | Slot 8; domain adaptation surrounds library-owned generic algorithms |
| DP-20 | Satisfied in inspected scope | Existing finite workers/permits and workspace admission; no new scheduler or unbounded retry |
| DP-22/23 | Satisfied in scope | F01 closed by complete scoped regression and required final checks; no performance claim |
| DP-24 | Satisfied in implementation scope | Public caller hard cut and generated declaration changes; ADR acceptance remains separate |
| PS-01/06/07/08/09/11 | Satisfied in inspected and targeted scope | Declared units/domain/smoothness, shared derivatives, explicit strategies and one model across analyses |
| PS-04/05 | Satisfied | Structural admission, distinct graph roles and constrained alternatives |
| PS-03/10/12/13 | Satisfied in scope | Original KKT/gap and conservation checks plus final public journeys; no broader property-dataset claim |
| PS-02 | Not changed | Existing selected material/property authority is retained; no new thermodynamic dataset qualification |

## 8. Library-leverage ledger

| Capability | Bespoke surface | Library/built-in and fit | Decision / remaining gap |
|---|---|---|---|
| Contextual selection | Domain requirements and deterministic policy | Native adapter capabilities and compiler facts | Keep one domain selector; a generic router cannot decide physical bounds/derivatives |
| Repeated solve | Coordinate/part compatibility and origin receipt | HiGHS clearSolver, primal/dual/basis starts; Ipopt warm-start controls; KINSOL initial vector | Keep thin provenance/transport; no custom iteration |
| Native optimality qualification | Original observations and accuracy policy | pounce-presolve recovery traversal; HiGHS original gap metrics and explicit `qp_regularization_value` | A recovered seed is not a KKT certificate; reject stronger claims when original checks fail |
| Recycle strategy | Selected graph, authored costs and port mapping | Existing graph algorithms, HiGHS exact tear MILP, KINSOL fixed point and Anderson | Keep independent acyclic witness; do not write another cycle breaker or Anderson solver |
| Conic analysis | Physical ports, shared policy and exact convexity evidence | Clarabel `CscMatrix`, cone enum, solver status and tolerance controls | Reuse library data types; no nonlinear-to-cone guessing or duplicate cone algebra |
| Hybrid dynamics | Model/event binding and support policy | Diffsol 0.16.2 event-time/reset sensitivity and mass consistency | Use built-ins; expose numerical time partials and refuse unsupported cases |
| Residual trial recovery | RAII/ABI, typed callback error mapping and model derivative projection | Pinned SUNDIALS IDAS, CalcIC, KLU, sensitivities and quadrature | Narrow smooth fixed-mass/index-1 route; no bespoke retry loop |
| Directional valve | Physical C2 transition law with declared pressure width | num-dual derivative propagation; shared provider/compiler contract | Domain formula is appropriate bespoke physics; existing libraries do not select the desired physical law |
| Fit qualification | Separate candidate/estimate semantics and rank policy | Native optimization and existing faer factorization | No new covariance, inverse or statistical framework; P14 owns sparse fit work |
| Boundary transport | Small typed document envelope and owned handles | serde, msgspec, PyO3, existing native API stub generator | Rust owns semantic admission; avoid a parallel Python analysis grammar |

Library investigation used native-solver-libraries, symbolica-faer-oximo and rust-graphs
skills, pinned local sources and Context7 for Diffsol, SUNDIALS, HiGHS and Clarabel. Those
references establish API fit, not this consumer's qualification. No new dependency pin,
licence restriction or generic algorithm is introduced by this packet.

The exact pounce-presolve 0.12.0 `PresolveTnlp::finalize_solution` source recovers
recorded reduction frames but does not establish the tested ordinary tightened-bound
multiplier's original complementarity. The existing independent checker is therefore
the authority. HiGHS documents QP Hessian regularization and its default; the consumer
test verifies both the default gap downgrade and explicitly smaller regularization.
See [HiGHS option definitions](https://github.com/ergo-code/HiGHS/blob/master/docs/src/options/definitions.md).

## 9. Alternatives

| Alternative | Meaning and custom code | Correctness / operational risk | Decision |
|---|---|---|---|
| Treat linked inventory as public capability | Small table | Feature unification can advertise routes the calling workflow cannot execute | Rejected; intersect explicit caller inventory and contextual requirements |
| Reuse implies warm start | Less explicit API | Sequence history silently changes root selection or basis state | Rejected; independent allocation and numerical-start controls |
| Diffsol plus custom retry loop | Fewer native bindings, more integration machinery | Application retries would own adaptive integration semantics | Rejected; use IDAS for its supported recovery profile |
| One general-purpose hybrid IDAS adapter | Larger model/ABI surface | Unqualified event sensitivity and layout transitions | Deferred by explicit refusal; retain Diffsol's qualified hybrid route |
| Library algorithms with typed domain adapters | Selector, physical law, provenance and transport only | Boundary contracts require focused controls | Selected and simplest viable design for the admitted operations |
| Hard valve clipping | Simpler expression | Hidden nonsmoothness changes derivatives and native behavior | Rejected; explicit positive-width C2 physical approximation |

No comparative performance result is claimed. Library ownership is justified by API fit
and mathematical contract, not an unmeasured speed claim.

## 10. Verification

| Claim | Evidence | Command / conditions | State |
|---|---|---|---|
| Routing/recovery/refusal | Tested | Filtered `just unit-package pse-backend-native`, native-solvers and force-validation | Seven selected tests passed, zero failed against zero baseline |
| Starts/cone/continuation/time fitting | Tested | Filtered `just unit-package pse-runtime`, native-solvers and force-validation | Five selected tests passed, zero failed |
| Public cone and explicit primal start | Tested | Focused `just py-unit` followed by complete `just py-test` with unit/component/integration selection and linked native environment | Two focused units and all 141 final Python tests passed |
| Latest native API compilation | Interface-checked | `just check-native-python` | Passed; compilation does not establish runtime correctness |
| Python/repository static checks | Tested | `just quality` with linked library environment; `just py-sync-native` generates actual API stubs | Passed; rebuilt boundary exercised by final complete Python suite |
| Complete supported behavior | Tested | Full default/native `just test`, doctests, rebuilt Python suite and `just plan14-python build/plan16-p07-p09` | 1,761 default and 1,817 native Rust tests, 141 Python tests and four public journeys passed; zero failures/skips in complete Rust runs; doctests passed |
| Generated/governance/docs | Tested | `just governance`, repaired `just codegen-relations-check`, `just clippy`, `just check`, ADR/docs/format/architecture checks | 95 governance tests and all required leaves passed; original failed aggregate exits are retained in the execution packet |

The zero baseline is required for every executed gate. Strict extra native-feature FFI
style Clippy retains the existing ADR-0087/M22 exclusion; ordinary default/no-default
Clippy passed. No full P18 performance, IDAES parity, distribution build or
other-platform acceptance is inferred from these checks.

## 11. Authority changes and exceptions

ADR-0093 was proposed before the affected extension. It remains proposed until the
decision-PR process and blueprint reconciliation are complete. This self-review does not
accept an ADR or edit accepted authority. Binding K1 follows Plan 14 library ownership;
K2 retains registry generation. Existing blueprint edits in the shared tree belong to
prior foundation work. No new SHOULD deviation or quality baseline is introduced.

## 12. Decision

**Accept scoped.** The examined P07–P09 architecture uses appropriate library owners and
explicit domain contracts. F01 is closed by the complete scoped functional regressions
and required final checks. Native optimality is granted only when original checks pass;
the tested presolve/QP limits remain visible. The existing native strict-lint exclusion,
proposed ADR decision route and P10–P18 boundaries remain. No unresolved MUST defect was
found within the supported behavior reviewed. This is implementation self-review, not
independent acceptance of the full Plan 16 target.

| Priority | Required action | Finding | Acceptance evidence |
|---|---|---|---|
| Correctness | Completed final Rust/Python/conserved-process and required static checks | F01 closed | Zero failures against zero baseline; commands and support limits in the execution Outcome |
