---
title: Plan 16 P05–P06 numerical contracts and shared facts
date: 2026-09-25
status: complete
scope: implementation and approved execution contract
evidence: Tested — targeted contracts, complete Rust regressions and linked Python journeys; scoped static boundary
---

# Plan 16 P05–P06 numerical contracts and shared facts

## 1. Scope, purpose and coverage

| | |
|---|---|
| Subject | [P05–P06 execution](../../plans/16-p05-p06-execution.md), math/compiler/structural/native/runtime/Python implementations |
| Standard | Core 2.0, process-simulator 1.0, pse-arrow binding |
| Tier · purpose | Design · conformance to the approved target and layered standard |
| Reviewer · date | Codex implementation self-review · 2026-09-25 |
| Decision | Accept scoped; see slot 12 |

The outcome is one resolved numerical policy, reversible native coordinates, attributable
shared mathematical facts and truthful final candidate use. The baseline had positional
tolerances, ignored declarations, competing affine interpretation and native/physical
acceptance conflation. Scope includes existing square, optimization, fitting and dynamic
workflows; conditional initialization consumes the same numerical owner.

Examined: precedence and units, conflict/refusal paths, symbolic constants and guards,
coefficient assumptions, normalization/duals/certificates, exact and numerical PSD,
structural identities, preparation identities and completion/table projections. Targeted
tests attacked invalid scales, lattice changes, cone conflicts, stale evidence, forbidden
cycles, ambiguous defaults and unavailable closure. Final native and Python journeys are
regression evidence, not proof of every backend status. Broader routing (P07), continuation
and recycle strategy (P08), dynamic strategy (P09), persistence (P12) and performance (P18)
are not claimed. This is not an independent reviewer assessment.

## 2. Authority and identity map

| Fact | Semantic type / identity | Authority / owner | Revision and update | Derived representations |
|---|---|---|---|---|
| Authored requirements and provider defaults | Target kind + semantic ID; requirement ID | Schema registry and selected sources | Model/case/analysis replacement | Generated Rust/Arrow/Python contracts |
| Numerical policy | `ResolvedNumericalPolicy`, framed key | `pse-math/src/numerics.rs:51` | Full selected sources, policy, quantities and actual coordinate scales | Native accuracy, tolerances, result provenance |
| Bound mathematical facts | `presolve::Facts`, structure/value key | Math; compiler tracked projections | Current coefficient and bound inputs | Class facts, coefficients, interval provider |
| Convexity evidence | Sealed `QuadraticEvidence`, matrix/assumption/policy key | `pse-math/src/convexity.rs:272` | Exact matrix, sign, coordinates, tolerance and resource limits | Backend admission and metrics |
| Structural block | `BlockId(scope, members)` | `pse-structural/src/incidence.rs:64` | Scope or membership changes identity; ordering alone does not | Conditional initialization schedule |
| Candidate assessment | Run/step, typed native/numerical/closure/usability fields | Completion in `workflow/run.rs:265` | Immutable once completed | Read-only Rust/Python views and tables |

Native algorithms are opaque library behavior under explicit options, callback contracts
and status translation. A native status is not the numerical or physical assessment.
Reordering independent declarations canonicalizes before numerical resolution; different
selected values and policies remain distinct. No new cache or publication authority is added.

**PS physical semantics**

| Element | Dimension and unit | Basis / convention | Validity | Authority |
|---|---|---|---|---|
| Nominal and absolute budget | Target quantity and canonical unit | Magnitudes: affine offsets do not apply | Finite positive scale; nonnegative tolerances; positive combined budget | Quantity registry + resolved target |
| Relative budget | Dimensionless | Multiplies frozen nominal, never the iterate | Finite nonnegative | Numerical requirement |
| Integer coordinate | Original lattice | Coordinate scale exactly one | Required incompatible substitution refused | Variable domain + resolver |
| Cone row block | Compatible quantity/unit within nonpolyhedral block | Common positive row factor | Conflicting required factors refused | Cone geometry + resolved policy |
| Closure | Original balance quantity; cumulative quantity separately | Authored inlet/outlet/work/heat convention retained | Missing required evaluation is unavailable | Balance declaration + completion assessment |
| KKT and gaps | Normalized minimization convention; independent budgets | Stationarity, complementarity, integrality and continuous/MIP gaps remain distinct | Native option admission and original measurements | Numerical policy and backend projections |

## 3. Contracts and invariants

| Invariant | Enforcement | Failure behavior | Evidence |
|---|---|---|---|
| One selected value per field | `numerics::resolve`, ranked candidates | Equal-priority conflicts and unknown targets fail | Precedence/unit tests |
| Reversible coordinate meaning | `normalization.rs:19`, native `transport.rs:56` and TNLP boundary | Dimensions, overflow and nonzero underflow fail | Root, coefficient, callback and dual recovery tests |
| No mixed-unit infeasibility scalar | `presolve/pipeline.rs:226`, per-bound normalized expansion | Unconfirmed proof cannot declare infeasibility; required reduction fails explicitly | Presolve certificate tests |
| Guards survive simplification | Shared original obligations and strict signs | Unestablished remains unknown; incompatible root domains refused | Guarded affine and strict-sign tests |
| Evidence belongs to current matrix | Sealed evidence validation and `Coefficients::matches_facts` | Stale assumptions/policy/coordinates refused | Convexity tests |
| Failed physical closure is visible | `workflow/numerics.rs:15` | Retained unusable candidate, or explicit qualified-unclosed use | Assessment and public encoding tests |

Absence, no required closure, unavailable closure, failed closure, unknown PSD, numerical
PSD and exact Gram proof are distinct. Equivalent units promise equivalent physical
budgets/acceptance within floating arithmetic, not identical native trajectories. Positive
congruence transports original quadratic evidence; it does not claim a fresh exact Gram
factorization of rounded normalized entries. No indefinite matrix repair occurs.

**PS well-posedness:** authored fixed/free/domain roles enter admitted case structure.
Existing DM/BTF analysis precedes conditional initialization; deficient partitions are
refused and report semantic members. Initialization consumes owner order and block IDs,
not a rebuilt positional interpretation. Stream cycles use connection witnesses and
forbidden decision IDs. This packet does not claim new routing for every problem class.

## 4. Derivation and execution

| Stage | Output / equality | Mechanism and dependencies | Reuse / determinism / cost | Effects and ownership |
|---|---|---|---|---|
| Resolve | Immutable numerical key and provenance | Domain precedence over selected declarations, provider bindings and quantity magnitudes | Preparation only; canonical target order; bounded selected target inventory | Pure preparation |
| Derive facts | Structure/value-specific facts | Symbolica derivatives/constants; retained obligations; pounce interval interpretation | Compiler tracked query; coefficients checked against same snapshot | Immutable owned products |
| Normalize / presolve | Scaled native contract + reversible maps | Positive diagonal adapters; library `PresolveTnlp` and recovery | Per prepared solve; bounded tapes and native workspace | Worker-owned native session |
| Assess curvature | Exact / numerical / indefinite / inconclusive | faer candidate LDLT + exact Gram check, optional residual-qualified EVD | Explicit byte/operation caps; serial algebra; cancellation before/after library calls | No mutation of Q |
| Structural projection | Identified ordered conditional blocks | Library matching/BTF/toposort/cycle witness | Semantic scope/membership IDs | Immutable graph-derived product |
| Complete | Original quality, closure and usability | Independent original evaluation, then one assessment | Once per completed run | Immutable retained result; table reads do not re-evaluate |

**PS numerical stages**

| Stage | Formulation / derivatives | Scaling / class | Status and tolerance checks |
|---|---|---|---|
| Admission | Original obligations retained; available order separated from requested prepared order | Bound shape and polynomial degree from admitted program | Unsupported smoothness/domain rejected before native entry |
| Native execution | Existing Symbolica/provider derivatives; no new finite-difference fallback | `x=Sx z`, `g/Sr`, `f/Sf`; native algorithmic scaling separate | Independent feasibility/KKT/gap options; no relaxed original bounds |
| Recovery | Original primal, gradients, row/bound duals, certificates and diagnostics | Original physical coordinates; integer lattice preserved | Original bounds/rows and guards checked; normalized KKT measurements separately reported |
| Dynamics / fitting | Existing differentiated models and native integrations | Parameter/state nominals; experiment aliases; integrated outputs and cumulative closure separate | Integrator local error control is distinct from frozen output/closure acceptance |

Graphs retain their relationship kind: equation incidence, conditional predecessors and
stream connectivity are separate. Serialization uses registry codecs; no new per-row FFI
protocol is introduced. Existing worker resource admission and result ownership remain.

## 5. Journeys

| Journey | Trace and result |
|---|---|
| Add a numerical requirement | One registry row, selected by target identity; same resolver projects it to each workflow. No backend-specific interpretation is authored. |
| Add a provider default | Explicit provider/output binding plus property/default selector; quantity mismatch or unknown output fails. Existing FeOS provider physics is unchanged. |
| Edit and re-solve | Numerical source changes alter source/prepared identity; coefficient changes invalidate bound evidence; changed scale invalidates native layout/warm compatibility. |
| Fit multiple experiments | Local rows/variables receive experiment aliases; global parameters retain one identity; the shared resolver feeds the assembled fit oracle. |
| Infeasible / ill-posed | Library structural or presolve proof is retained with model identities; tolerance-incompatible infeasibility is not promoted to a certificate. |
| Dynamic integration | State scaling, integrated-output budgets and cumulative closure are distinct; existing event/partial trajectory semantics remain. |
| Boundary round trip | Completed assessments and resolved provenance pass generated Arrow codecs and the Python workflow; inspection reads retained results. |
| Cancellation / failure | Existing owned worker cancellation joins execution; curvature checks bound dense work and do not claim interruptibility inside faer. No provisional result becomes a committed success. |

Recycle convergence redesign, arbitrary provider envelope qualification and new dynamic
event strategies are outside this change. Their existing behavior is exercised by the
regression suites only where those suites select it.

## 6. Gates

| Gate | Verdict | Evidence / scope | Required action |
|---|---|---|---|
| G1 Authority | Pass | One resolver; legacy positional public tolerances removed | None |
| G2 Semantic fidelity | Pass | Explicit units/coordinates, source provenance, matrix assumptions and block IDs | None |
| G3 Validity | Pass | Resolver, native shape/domain admission and original assessment enforce contracts | None |
| G4 Hidden behavior | Pass | Resolution/evaluation timing explicit; native options recorded; inspection pure | None |
| G5 Consistency and recovery | Pass | Existing ownership/publication model retained; new assessment immutable | No new persistence claim |
| G6 Transformation and reuse | Pass | Checked transport and complete evidence identities | None |
| G7 Truthful capability claims | Pass | Targeted and final suites passed; unsupported native lint/release claims excluded | Preserve the stated qualification boundary |
| G8 Library leverage | Pass | Algebra, interval propagation, graph algorithms, factorization and solves remain library-owned | None |
| PS-G1 Physical model validity | Pass | Magnitude units, explicit property bindings and distinct closure budgets | No new property dataset claim |
| PS-G2 Structural well-posedness | Pass | Shared scope/member identities, deficiency rejection, cycle witnesses | None |
| PS-G3 Numerical trustworthiness | Pass | Original checks, exact/approximate distinction and final native/Python journeys passed | Broader status qualification remains P07 |

## 7. Findings

| ID | Finding | Principles · gate | Evidence / gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| F01 — closed | Targeted evidence alone did not close execution | DP-22/23 · G7, PS-G3 | Final default/native suites, Python journeys and required static leaves now pass | Premature closure would have overstated evidence | Completed the final stage after functional implementation | Slot 10 and execution Outcome |

No additional unresolved implementation defect was found in the examined contracts.
Problems found during implementation were repaired before this review decision: builtin
Symbolica constants mistaken for parameters; a remaining positional test caller; and
round-trip certificate/diagnostic transport omissions. Their regressions remain in scope.

| Principles | Verdict | Mechanism / scope |
|---|---|---|
| DP-01, DP-05, DP-06, DP-17 | Satisfied | Registry/source/math/native/completion owners; no parallel numerical authority |
| DP-02, DP-03, DP-21 | Satisfied | Typed target/outcome distinctions, enforced admission and attributable proof/provenance |
| DP-04, DP-09 | Satisfied | Canonical source keys, consumed-value evidence and scope/member block identity |
| DP-07, DP-12 | Satisfied | Directed relationship-specific graphs and library cycle witnesses |
| DP-08, DP-11, DP-15 | Satisfied | Checked coordinate maps, exact versus approximate evidence and native contract tests |
| DP-10, DP-18, DP-19 | Satisfied | Prepared immutable policies and completion-owned results; no inspection evaluation |
| DP-13, DP-14, DP-16 | Satisfied | Slot 8; custom code expresses domain policy and transports library contracts |
| DP-20 | Satisfied | Existing worker permits plus explicit curvature/tape budgets; no new executor |
| DP-22, DP-23 | Satisfied | Complete suites and repaired leaves recorded with zero baseline; native strict-lint exclusion explicit |
| DP-24 | Satisfied | Generated contracts, hard-cut callers and proposed ADR extensions |
| PS-01, PS-03, PS-06, PS-07 | Satisfied | Physical magnitude semantics, original closure, guards and derivative/scale contracts |
| PS-04, PS-05 | Satisfied | In-scope conditional partition and structural diagnostics preserve owner products |
| PS-08 | Satisfied in touched preparation | Immutable staged coordinates; continuation/recycle strategy remains P08 |
| PS-09 | Satisfied in touched adapters | Existing class admission and native solver algorithms; broader qualification P07 |
| PS-10, PS-12 | Satisfied in scope | Distinct native/numerical/physical results and final public journeys; no broader native-status claim |
| PS-11 | Satisfied | Same numerical resolver across steady, fitting, dynamics and initialization |
| PS-02, PS-13 | Not applicable to new physics | No property or unit model introduced; existing model journeys provide regression evidence |

## 8. Library-leverage ledger

| Capability | Bespoke surface | Library / built-in used | Fit and gaps | Recommendation |
|---|---|---|---|---|
| Symbolic classification | Admitted-expression projection and provenance | Symbolica constants, differentiation, normalized atoms | Own policy must retain original guards; symbol presence alone is not a variable test | Keep shared `Facts`; `is_constant` owns constant recognition |
| Interval/presolve | TNLP adapters and tolerance-qualified proof mapping | pounce-presolve FBBT, `PresolveTnlp`, elimination/recovery | Library does not own physical tolerance precedence | Keep library reductions; retain per-bound domain budgets |
| Convexity | Evidence wrapper, matrix/assumption identity, qualification | faer serial LDLT/EVD; existing exact Gram verifier | Numerical decomposition alone cannot establish an exact rational certificate | Keep exact default and explicit approximate/inconclusive path |
| Structural analysis | Semantic member/scope mapping | Existing DM/BTF + petgraph/rustworkx topological order and cycle witness | Library indices cannot be durable domain identity | Keep thin identity projection; no custom graph algorithm |
| Solver termination controls | Domain accuracy projection | Ipopt/POUNCE acceptable/bound options; KINSOL strict signs/scales; HiGHS gaps; Clarabel tolerances/equilibration | APIs have different supported bounds/scaling semantics; refuse conflicts | Use built-ins; qualify additional statuses/routes in P07 |
| Source/result boundaries | Registry declarations and completed rows | Existing serde and generated Arrow/Python adapters | No runtime inference can replace authoritative schema identity | Extend existing generation; no new generator |

Reference selection used the native-solver-libraries and symbolica-faer-oximo skills,
exact pinned local source and Context7 Symbolica documentation. No library was excluded
on licence or lack-of-consumer grounds. No dependency upgrade was needed. A generic
configuration framework would add another interpretation layer without replacing the
domain decisions above.

## 9. Alternatives

| Alternative | Meaning / custom code | Correctness risk | Cost evidence | Decision |
|---|---|---|---|---|
| Keep positional backend tolerances | Each caller interprets targets, units and defaults | Reordering or mixed units changes acceptance | None | Rejected |
| Delegate all scaling to native heuristics | Less adapter code, but loses authored coordinate requirements | Heuristics cannot own physical acceptance or integer/cone rules | None | Rejected as sole authority; retained separately where supported |
| Library algorithms + explicit model policy | Domain resolver and reversible adapters only | Boundary transformations require tests | No performance claim | Selected; simplest viable library-owned design |
| Automatic PSD clipping / diagonal shift | Replaces Q and creates another objective | Silently solves a different problem | None | Rejected |
| Exact proof only | Existing exact Gram path | Safe but refuses otherwise useful approximate admission | None | Retained default; explicit numerical assessment is optional |

## 10. Verification

| Claim | Evidence | Test / conditions | Current result |
|---|---|---|---|
| Resolution and shared facts | Tested | Filtered `just unit-package pse-math` numerical-policy, convexity, facts and admitted-transcendental tests; force-validation | Six passed, zero failed against zero baseline |
| Structural semantics | Tested | Filtered `just unit-package pse-structural` identity, conditional schedule and forbidden cycles | Three passed, zero failed |
| Native transport/options/proofs | Tested | Filtered `just unit-package pse-backend-native`, native-solvers + force-validation | Six passed; separate root and coefficient/certificate recovery checks each passed |
| Workflow policy and fit contract | Tested | Filtered `just unit-package pse-runtime`, native-solvers + force-validation | Five passed, zero failed |
| Public process journeys | Tested | `just plan14-python build/plan16-p05-p06`, linked editable extension, local Linux | Four passed, zero failed |
| Complete Rust regression | Tested | `just test`; same with runtime native-solvers and conformance native-acceptance features; force-validation | 1,752 default and 1,797 native passed; zero failures/skips |
| Complete Python surface | Tested | `just py-test` with unit/component/integration marker selection; Python 3.14.7, rebuilt native extension and fresh store | 139 passed; four public acceptance journeys also passed |
| Required static surface | Tested | Default/no-default Clippy, doctests, governance/generated comparisons, format, quality, ADR/docs/manifest; affected leaves rerun | Passed within the packet's recorded boundary; strict native-feature Clippy is not claimed clean |

No unit or property physics was changed by P05–P06. Existing FeOS, vessel, fitting and
physical NLP journeys establish only their exercised regression scope. No new IDAES
numerical parity, performance, release or other-platform claim follows from these tests.

## 11. Authority changes and exceptions

Proposed ADR-0088/0089/0090 were extended before affected code for numerical meaning,
identity and result vocabulary. Their decision-PR acceptance remains pending. This review
does not amend the blueprint or accept an ADR. Binding conflicts K1/K2 are resolved using
Plan 14's library ownership and existing registry generation, as the binding directs.
No new SHOULD exception is requested. The existing ADR-0087 / Plan 14 M22 design-stage
boundary excludes strict native FFI documentation/style lint qualification. The extra
native Clippy attempt reported 140 backend test-build findings against baseline zero;
it is not a successful gate. Ordinary default/no-default Clippy passed. No lint was
suppressed or baseline added. No new crate, dependency family or pin is introduced.

## 12. Decision

**Accept scoped** — the examined P05–P06 architecture and its complete exercised
functional surface meet the approved target. F01 is closed by final qualification.
The existing native strict-lint exclusion under ADR-0087 remains; this decision does
not claim a clean strict native-feature Clippy gate, release readiness or P07–P18
completion. No unresolved MUST defect remains in the supported behavior reviewed.

| Priority | Change | Finding | Acceptance evidence |
|---|---|---|---|
| Correctness | Completed native/public regression and required generated/static checks | F01 closed | Execution packet Verification and Outcome |
| Library use | Retain library-owned algorithms and typed domain adapters | None open | Slot 8 and targeted tests |
| Cost | Keep dense curvature work resource-bounded | None open | Explicit limits; performance remains P18 |

The next ordinary numerical extension is a registry/source declaration and, where a new
native option is involved, one adapter projection with its boundary tests.
