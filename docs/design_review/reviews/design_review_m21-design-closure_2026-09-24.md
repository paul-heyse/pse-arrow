---
title: M21 implementation and design closure
status: complete
date: 2026-09-24
scope: Plan 14 M21 implementation mechanisms; not M22 qualification
adrs: [ADR-0082, ADR-0083, ADR-0084]
---

# M21 implementation and design closure

## 1. Scope, purpose and coverage

| Field | Value |
|---|---|
| Subject | [M21 execution](../../plans/14-m21-execution.md), touched compiler/provider/solver/workflow boundaries and their current consumers |
| Standard | Core v2.0, process-simulator v1.0, [repository binding](../design_principles/binding/pse-arrow.md) |
| Tier · purpose | Change · conformance to the approved Plan 14 target |
| Reviewer · date | Implementation agent, self-review · 2026-09-24 |
| Decision | Accept the bounded implementation mechanisms; no M22 or independent gate acceptance |

**Implemented:** M21 removes remaining obsolete production surfaces and closes the
selected policy, property-envelope, conservation, structural-admission and evidence
contracts. Analysis modes considered are square simulation, NLP optimization,
coefficient/conic execution, fixed-mass ODE/index-1 dynamics and scoped fitting.
Heater/recycle, flash/separation, vessel events and parameter-estimation fixtures are
current consumers. Their complete scientific and runtime journeys remain M22.

The review examined declarations, identity framing, provider trial/NPT admission,
balance projection and observations, structural entry points, implicit response
checks, generated contracts, current callers and evidence authentication. Negative
controls attack duplicate equation ownership, unpaired transfers, changed mode
contributions, duplicate accumulation, undeclared reset impulses, invalid envelopes,
unmatched equations and altered source/native evidence. Authoring and generic ID
controls exercise the hard deletions. Compilation covers M22 bodies without running
them. Native process convergence, installed journeys, reference comparisons,
publication faults, RSS/performance and whole-plan static/governance are **not_run**.

Known conflict K1 is resolved for this scope by the maintainer-approved Plan 14 and
proposed ADR-0082–0084, as the binding directs. The older blueprint is not silently
rewritten or treated as evidence of this implementation. K2's surviving registry
projections remain authoritative for declarations/results. Formal ADR acceptance and
blueprint reconciliation remain M22 work through the established decision-PR route.

## 2. Authority and physical semantics

| Fact | Owner and revision boundary | Derived consumer |
|---|---|---|
| Numerical policy | `pse_math::binding::guarded_real_policy`; compiler-derived, versioned semantics | `BodySpec` and Salsa semantic products; no caller-authored hash |
| Provider window | Generated `native_providers.envelope`, checked `StateEnvelope`, bounds/provenance framed into provider identity | Coherent FeOS state and separate NPT initialization |
| Physical balance | Generated `physical_balances` row, source IDs, typed roles, mode selectors, transfer identity and provenance | Temporary compiler rows and independent original-output closure |
| Dynamic conservation | Same balance plus conserved state, canonical tolerance and explicit event impulses | Diffsol integrated outputs, segment carry and sample checks |
| Solve outcome | Native status, candidate, mathematical quality and physical checks remain distinct | Generated result families; missing physical evaluation never becomes accepted |
| Implementation evidence | Actual version-3 runner report, exact witnesses, current source/native bytes | Minimal local development pointer and implementation barrier |

| Physical element | Unit/basis/convention | Envelope or validity | Authority |
|---|---|---|---|
| Ternary FeOS state | K, mol/m³, Pa; ordered methane/ethane/propane fractions | Explicit finite bounds on all coordinates; intrinsic positivity/interior restrictions also apply | Provider declaration and `pse-kernels/src/envelope.rs` |
| Caloric quantities | Existing typed FeOS/DIPPR reference; no formation-enthalpy claim | Same coherent state and envelope; empirical accuracy unqualified | Existing physical quantity/reference registry and FeOS package |
| Steady flow/energy closure | Canonical quantity units; contribution roles determine sign | Per-balance positive tolerance; matching internal transfer pairs | `authored.physical_balances` |
| Accumulation and reset | Canonical conserved quantity; rate is its physical time derivative | Separate native output integration tolerances and conserved-state tolerance | Dynamic/balance declarations and simulation profile |

## 3. Enforcement and well-posedness

`crates/pse-runtime/src/workflow/balances.rs:59` (`project`) rejects duplicate balance equations,
invalid source/output references, overlapping mode selections, duplicate conserved
state owners, missing differential-state bindings and unpaired internal transfers.
Generated rows pass through the existing physical compiler; `crates/pse-runtime/src/workflow/model.rs:147` (`freeze`)
charges mode expansion before allocating it. Values originate in original body
outputs, not in solver-scaled aggregate residuals.

`crates/pse-kernels/src/feos.rs:209` (`initialize_npt`) and its worker check all input axes before state-cache reuse, computes
pressure from the coherent state even when pressure is not requested, and checks
both input and resulting NPT state. `OutsideEnvelope` remains a recoverable trial
error. A declared operating window is not an empirical validity certificate.

`crates/pse-backend-native/src/structural.rs:20` (`admit`) consumes complete matching/DM results.
Roots require a square complete matching; NLP requires matched equalities and allows
optimization freedom. Diagnostics preserve original row/column IDs. Original
residual support governs fixed-point/Picard admission. Dynamic algebraic-partition
analysis remains separate from flowsheet topology. Coefficient/conic routes retain
native presolve. No structural result certifies numerical nonsingularity.

`crates/pse-runtime/src/workflow/fitting/oracle.rs:773` (`check_response`) uses faer multiplication
and norms to verify the scaled implicit solve before unscaling: backward error is
bounded by `64 * max(1,n) * f64::EPSILON`. Zero denominators and nonfinite residuals
are handled explicitly. SVD remains the existing scoped rank diagnostic.

## 6. Gates

Verdicts below concern the bounded mechanism and truthful implementation claims
above. The eleven independent, complete-system M22 verdicts are still outstanding.

| Gate | Verdict | Evidence and scope | Required action |
|---|---|---|---|
| G1 Authority | pass | Generated balances own terms; projected equations reject duplicate authorship; compiler owns policy | M22 checks full round trips |
| G2 Semantic fidelity | pass | Source IDs, signs, physical typing and canonical scaling survive projection; mode/event controls | M22 compares complete process references |
| G3 Validity | pass | Provider bounds, original-equation matching, explicit dynamic/output tolerances and response error checks | M22 convergence and numerical qualification |
| G4 Hidden behaviour | pass | Native state/evaluator effects remain worker-owned; envelope pressure calculation and output integration are explicit | M22 complete runtime observations |
| G5 Consistency and recovery | pass | Invalid trial recovery, event impulse refusal and nullable failed physical checks; existing publication owner retained | M22 faults/cancellation/reopen |
| G6 Transformation and reuse | pass | Policy/envelope/source terms enter identity; compiler products use existing Salsa; stale receipts/native bytes refused | M22 clean/reused process equivalence |
| G7 Truthful capability claims | pass | Removed grammar has no fallback; physical inference vocabulary is not numerical availability; evidence distinguishes units from process qualification | Independent current-source review at M22 |
| G8 Library leverage | pass | strum vocabulary, Symbolica/Numerica math, FeOS properties, pounce structure, faer response check and Diffsol integration | Measure complete cost at M22 |
| PS-G1 Physical consistency | pass | Enforced declaration windows; signed balances; independent raw-term and integrated-accumulation closure | Empirical validity and full physical cases remain unqualified |
| PS-G2 Well-posedness | pass | Class-aware complete original matching; NLP freedom preserved; dynamic algebraic partition remains distinct | Full ill-posed/convergence journeys at M22 |
| PS-G3 Numerical integrity | pass | Existing exact derivatives/guards, native output tolerances, reset accounting, explicit response error and separate outcome evidence | M22 scientific/runtime qualification |

## 7. Findings and principle verdicts

No open MUST defect was identified in the bounded M21 mechanism review. The following
concrete defects were corrected during implementation; they are not waived findings.

| ID | Finding | Principles · gate | Evidence / consequence | Correction | Verification |
|---|---|---|---|---|---|
| F01 | Authored policy and unsupported parser variants outlived the supported math contract | DP-01, DP-15, DP-24 · G1/G7 | Source hash could claim policy without defining executable semantics; removed functions still parsed | Compiler-owned policy; strum-supported vocabulary; grammar/callers/snapshots updated | Authoring units and exact native compiler selection |
| F02 | Property and conservation contracts lacked enforced source ownership | PS-02, PS-03 · PS-G1 | A valid-domain iterate could leave the declared operating window; a small residual could be mistaken for conservation | Envelopes, contribution authority, raw observations and native flux integration | Envelope, contribution and integrated-event controls |
| F03 | Structural/response checks needed the original problem and scaled numerical evidence | PS-04, PS-07 · PS-G2/3 | A splitting/map support or tiny unscaled element could conceal deficiency/error | Original matching and scaled faer normwise response check | Structural admission and implicit-response controls |
| F04 | Sealing trusted classified summaries more than executed artifacts | DP-22, DP-23 · G7 | Altered binaries or handwritten pass summaries could authorize stale work | Existing collector now authenticates exact units, actual reports and native bytes during sealing and continuation | Actual-runner and native-identity rejection controls |
| F05 | Balance mode expansion and repeated fixture blocks needed explicit ownership | DP-06, DP-20 · G1/G3 | Authored size did not account for expansion; copied process blocks omitted balances | Pre-allocation projection charge; fit physical report extent; fixture balance remapping | Mode projection unit; acceptance/measurement consumers compile |

| Applicable principles | Verdict | Enforcement / evidence |
|---|---|---|
| DP-01, DP-05, DP-06 | Satisfied | One source family owns contributions; compiler owns derived equations and policy |
| DP-02, DP-03, DP-04 | Satisfied | Generated typed roles, finite checked intervals, semantic identities and explicit refusal |
| DP-07 | Satisfied | Transfer pairing preserves roles; original incidence stays distinct from physical topology |
| DP-08, DP-11 | Satisfied | Canonical physical conversion, explicit error/tolerance and guarded-real contracts |
| DP-09, DP-10 | Satisfied | Existing Salsa dependency path and prepared library artifacts retained; no parallel cache/compiler |
| DP-12 | Satisfied | Existing bounded native events/steps; integration segment carry is explicit |
| DP-13, DP-14, DP-16, DP-17 | Satisfied | Ledger below: domain adapters around library-owned generic operations |
| DP-15 | Satisfied within admitted profile | Pinned native interfaces and focused units; no broader library coverage claim |
| DP-18, DP-19, DP-20 | Satisfied within logical limits | Worker-local effects; expansion/report admission; native RSS qualification excluded |
| DP-21, DP-22, DP-23, DP-24 | Satisfied | Attributable physical errors, hard schema cut, exact runner evidence and explicit M22 boundary |
| PS-01, PS-02, PS-03 | Satisfied for declared contracts | Physical compiler, envelope admission and separate conservation results |
| PS-04, PS-05 | Satisfied | Complete original matching plus existing algebraic partition |
| PS-06, PS-07 | Satisfied | Compiler policy, exact library derivatives, native integration tolerances and scaled response checks |
| PS-08, PS-09, PS-10 | Satisfied within existing profiles | Library initialization/solver lifecycle, explicit class checks and separate outcome evidence |
| PS-11, PS-12 | Satisfied | Same balance/model sources across solve/dynamics/fit; physical result family |
| PS-13 | Satisfied for this package's shared checks | Common controls and M22 reference bodies exist; empirical qualification is explicitly not claimed |

## 8. Library-leverage and consumer ledger

| Capability | Adapter / retained consumer | Library operation and fit | Disposition |
|---|---|---|---|
| Function names | `pse-math/src/functions.rs` and schema builder | strum derives parsing, spelling and variants from one enum | Retain thin vocabulary; delete unsupported grammar |
| Arithmetic / differentiation | Typed compiler, `pse-math` guarded regions | Symbolica atoms, differentiation/evaluators; Numerica arithmetic jets | Retain; no new evaluator or AD IR |
| Property state | `pse-kernels/src/feos.rs` | FeOS state, pressure/NPT and num-dual derivatives | Retain coherent state; envelope is domain admission, not replacement thermodynamics |
| Integrated physical flux | `pse-backend-native/src/dynamics/integrator.rs` | Diffsol `integrate_out`, `out_rtol/out_atol`, `interpolate_out` | Use library error control/interpolation; only event/segment accounting is ours |
| Matching / DM / BTF | Existing `pse-structural::incidence`, original native oracle boundary | pounce-presolve matching/partitions and existing semantic witness mapping | Retain; no bespoke matching or rank substitution |
| Implicit response | Fitting oracle | faer LU/SVD, matrix products and norms | Retain; domain check verifies numerical result |
| Incrementality | `pse-compiler::workspace`, runtime artifact consumers | Salsa tracked inputs/queries/backdating; DataFusion-owned bounded runtime cache | Retain pure facts and separate worker ownership; no solver state in Salsa |
| Physical graphs | Existing flowsheet, package topology, conditional blocks, tears | petgraph/rustworkx and qualified existing pounce projections | Retain on current caller evidence; old Plan 13 algorithms are not reintroduced |
| Data/storage | Source loading, inspection, result retention/publication | Arrow/DataFusion plus exact pinned Delta operations | Retain boundary roles; no relational transport inside callbacks |
| Generic IDs | `pse-ids` framing used by current body/source/provider identities | BLAKE3 framed identity | Delete orphaned specialized constructors and ordinal wrapper; retain generic contract |
| Physical-only vocabulary | `pse-quantity` inference, registry metadata/inspection | Physical type algebra independent of numerical evaluation | Retain including smoothing/weighted-mean compatibility metadata; it does not advertise executable functions |
| Convert/Broadcast authoring | Parser, render, walk, source binding/inspection | Existing editing contract | Retain explicit source operations; numerical lowering refuses them |
| Parity reference | `python/pse/parity` isolated IDAES/Pyomo environment | Reference oracle only | Retain actual reference consumer; delete orphaned API probe and active regeneration |
| Evidence collection | Existing validation runner, Nextest/pytest/unittest reports | Existing execution, XML/report and native provenance mechanisms | Extend exact unit mode; no second status schema or generated review approvals |

The simpler library-owned alternative is the selected design: declare physical terms
once, compile through the existing math path, integrate outputs in Diffsol and use
faer/pounce for their existing numeric/structural responsibilities. A host quadrature,
second balance evaluator, JSON vessel adapter, parallel Salsa compiler or new evidence
framework would duplicate meaning without improving these contracts. No speedup is
claimed without M22 measurements.

## 11. Authority route

Proposed ADR-0082–0084 record these changes before implementation. Accepted ADRs and
the protected blueprint remain untouched. M22 must reconcile their formal status via
the decision/design PR route and obtain independent G1–G8 plus PS-G1–PS-G3 decisions
from reviewers who are not implementation authors. This file cannot satisfy those
independent artifacts.

## 12. Decision

**Accept the M21 implementation mechanisms at the stated scope.** Targeted evidence
is recorded in the [execution packet](../../plans/14-m21-execution.md). This decision
permits proceeding to qualification; it does not qualify a process simulator.

| Priority | Remaining work | Finding / scope | Acceptance evidence |
|---|---|---|---|
| Correctness | Execute M22 native process, scientific, installed Python and publication journeys; repair failures | Complete-system behavior outside this review | Q01–Q17 in exact declared profiles |
| Library fit and locality | Independently review the final implemented source after functional qualification | All eleven M22 gates | Authenticated reviewer artifacts, no open MUST defects |
| Cost | Measure complete cold/warm process behavior and resources | No performance claim established here | Q18 and guarded measurement profiles |
| Governance | Close full static/generated/doc gates and formal ADR/blueprint reconciliation | Whole-plan closure | M22 zero-baseline qualification receipt |
