---
title: "25m: Integrated solve pipeline"
status: in-progress
date: 2026-10-03
adrs: [ADR-0083, ADR-0093, ADR-0144, ADR-0145, ADR-0150, ADR-0152, ADR-0153, ADR-0154, ADR-0155, ADR-0156, ADR-0157]
review_sources: [docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md, docs/design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md]
scenario_sources: [docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#scenarios, docs/design_review/design_principles/binding/pse-arrow.md#architecture-scenarios]
---

# 25m: Integrated solve pipeline

## Purpose, boundary and ownership

Implement the fully integrated acceleration/globalization pipeline specified by the
[earlier review](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md)
and refined by the
[integrated review](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md).
Retain T1–T10, RR1–RR10 and all applicable M1–M18 families, including M1b. Their
established mathematics is assumed under its stated conditions. Source/interface fit,
correct implementation and preservation of the original answer contract remain obligations.
Method inclusion does not require benchmarking, exhaustive combinations or an experimental
ranking campaign.

The result is a shared numerical strategy that composes start preparation, prediction,
derived systems, native profiles, retained information, observations and transitions.
It minimizes unnecessary expensive work without replacing authored physics or granting
results from auxiliary success. A direct attempt with no preparation remains a complete,
minimal strategy.

This companion owns the adopted F01–F17 and IP01–IP12 finding dispositions below.
[Plan 25](25-design-remediation.md) links this owner; it retains its other findings.
[Plan 25k](25k-integrated-qualification-and-closure.md) remains the sole owner of the
assembled qualification, dev-profile measurements and closure campaign. This plan owns
functional packet progress, not another full campaign. Its functional handoff enables K;
its eventual closure consumes K's evidence, avoiding a dependency cycle.

The boundary includes direct mathematical execution, modeling cases, initialization,
recycles, continuation, studies, fitting/profile likelihood, horizons, shooting and dynamic
initialization. Statistical targets, control application, integration time stepping and
durable occurrence/effect policy retain their scientific and lifecycle owners. Discrete/global
execution keeps its existing guarantees; numerical rungs cannot transfer an auxiliary
global certificate to the original problem.

Existing architecture owners are blueprint §§5, 7, 14–19, 20 and 23.2.
Core 3.3 / ProcessSimulator 1.3 govern the authoring assessment. Reviews supply evidence;
the required decision route below supplies authority for changed contracts.

## Current baseline and foundation assessment

Authoring baseline: clean HEAD ea402d56b32b2d80865f03775754f65dc3042766 on 2026-10-03.
The formerly uncommitted Plan 25k work and both reviews are now present in that commit.
Historical receipts keep their original conditions; committing them does not qualify new work.

**Implemented, source-inspected:** shared absolute cancellation/deadline propagation;
demand-indexed executable support; complete original structural incidence; contextual
class/representation/settings routing; original-coordinate native assessment and composed
workflow permission; worker-owned native sessions; complete competitive nonlinear-root
exclusion and regular selected charts. Retain these foundations.

The material changes are:

- Fix truthful outcomes/settings and erased profile-chain failures before transitions consume them.
- Extend the existing deadline with strategy accounting and phase/scope distinctions.
- Separate capability evidence from trajectory history and original permission from artifact retention.
- Extend selected-root products with consumed sheet continuity, useful proof promotion and numerical reuse.
- Compose derived families and actual oracle derivatives through existing preparation owners.
- Move duplicated numerical policy into the shared strategy while preserving distinct workflow policy.
- Retain actual reusable root/KKT/setup objects; application retention alone is insufficient.

The assessment reuses both reviews rather than reopening the whole subsystem. Selective
current-source inspection confirms the consequential seams: MathService::admit_step,
NativeSession::step, Outcome::{Native,Constant,Rejected}, routing::Context::refusals,
workflow::numerics completion, fitting/profile::Chains and implicit_regimes::evaluate_certified.
The specialized compiler derived-bound lowering is not the new numerical derived-family owner.

Credible changes this design accommodates are a new method profile, replacing a native
provider, composing another study consumer, re-solving after value versus structure changes,
and testing policy without native/store startup. Each changes its appropriate owner rather
than requiring workflows to reinterpret starts, failure or qualification.

## Combined target and contracts

### Responsibilities and execution flow

| Responsibility | Authoritative owner | Product consumed elsewhere |
|---|---|---|
| Declared strategy, mechanism/start/branch kinds and result vocabulary | pse-model and existing pse-schema declarations where generated consumers need them | Validated domain policy and generated representations |
| Original/derived preparation, incidence, demand and mathematical composition | pse-compiler / pse-math | Immutable prepared families, bound steps, reconstruction and evaluator support |
| Native capabilities, method settings, callback semantics and reusable foreign objects | pse-backend-native | Admitted profiles, truthful execution observations and keyed native products |
| Strategy resolution, transitions, start admission and task driver | pse-runtime::math | Pure resolution/transition products, effectful execution and one trace |
| Scientific targets, control, integration and durable effects | Existing workflow owners | Bound targets and independent assessment obligations |
| Original candidate permission | Existing shared completion owner | One composed result/seed decision, consumed by every workflow and strategy |

Concept names below specify meaning, not a requirement for one struct, service or crate per
paragraph. Keep the strategy in the existing crates. Native foreign-binding crates follow
their own decision route.

A solve task carries the immutable original identity/binding, frozen final numerical policy,
declared strategy and start permission, existing ExecutionScope, retained-product inventory,
executor and assessment operation. Pure resolution consumes snapshots of capabilities,
problem facts and available work; it never reads clocks or acquires evaluators/native state.

Resolution produces a finite strategy with preparation actions, execution forms/profiles,
derived-system bindings, start rules, accuracy demands, allowed transitions and caps.
An effectful driver prepares/adopts starts, executes admitted work, assesses outcomes, charges
the ledger and invokes the pure transition. Every actual rung has its own route/profile,
original-or-derived identity and typed reason. Preparation, report-less refusal, constant
evaluation and failed native execution are all representable events.

### T1/T2/T7 — strategy, transitions and assessment seam

Mechanism descriptions declare applicability, required information/support, composition
position, owned numerical operation, triggers and abandonment rules. They may be preparation,
an in-attempt native profile, an execution form or a later recovery rung; a literal
direct-first escalation list is insufficient. The planner orchestrates descriptions and
existing contextual routing rather than owning method mathematics or a new backend ranking.

Preserve the earlier review's escalation families and triggers with the following correction:
routing::Context::refusals contains established capability/representation incompatibilities,
never ordinary numerical trajectory failures. Attempt history is a separate input.
A failed trajectory can therefore permit another profile on the same backend.

| Observed condition | Transition meaning |
|---|---|
| Original completion grants permission | Finish under that permission; retain only separately admitted products |
| Lawful numerical trajectory failure, stationary non-root, restoration/line-search failure | Take the declared recovery/subdivision, if its contract and remaining work permit |
| Iteration cap without declared stagnation evidence | Record a limited attempt; do not invent a method-failure conclusion |
| Certified original infeasibility | Stop after independent certificate validation |
| Invalid required model/contract/capability, task exhaustion, cancellation, infrastructure failure or panic | End the affected task; no automatic retry |
| Optional preparation unavailable, or optional mechanism slice exhausted | Record its typed reason; an already permitted base route may remain available |
| Derived/intermediate success | Assess auxiliary use; it is not original result permission |

Explicit backend selection is preserved. Changing profile, start or derived system is lawful
only within the declared permissions; changing backend requires the selected alternatives.
An automatic strategy is deterministic from its facts and policy, not a promise of universal
optimal performance.

The executor and assessment are separate injected operations. Production executes through
MathService and the existing backend table; policy tests use a scripted executor and
explicit assessment evidence without native or storage initialization. The assessment returns
the composed original candidate decision plus auxiliary/artifact-specific permission.
Do not replace it with SolveReport success, Quality::feasible or AssessedPoint::accepted.

Numerical attempt abandonment is distinct from user cancellation. It stops the trajectory
through native-supported callbacks and records its reason, without poisoning cancellation
for a permitted subsequent rung. An indivisible call may finish late; check the enclosing
scope before retaining or using its evidence.

### T3/T5 — starts, products and validity

Resolve the entry start once from the existing request:

- NoPriorStart uses the specification and excludes inherited solutions, predictors and
  native start payloads. Fresh construction within this task requires its own declared mechanism permission.
- Explicit requires the supplied admitted seed; do not substitute a prediction or predecessor.
  A declared later recovery may derive from that attempt only when its strategy permits it.
- PreviousAccepted uses a compatible predecessor with result permission, or the existing
  specification fallback when absent. Seed-only is not PreviousAccepted.
- Study edges that explicitly permit seed-only retain that separate permission.
  They do not weaken sequence or block-commit semantics.

Automatic mechanisms cannot override these entry rules. Subsequent proposal origins allowed by
the declared strategy include accepted, predicted, auxiliary, partial, modified-specification
and surrogate starts. Repair/projection is a declared transformation with provenance, not a
silent clamp. Screen finite values, required bounds and evaluability before native execution;
an equation-infeasible but evaluable point may still be a lawful start.

Distinguish semantic proposals from native payloads. A semantic point can cross adapters after
original screening; bases, working sets and native sessions require their stronger compatibility.
Product keys include the dependencies actually consumed: original structure/binding, resolved
policy and normalization, point/parameters, derivation, source/build/profile, order/accuracy
and branch evidence. Symbolic-pattern compatibility is different from fresh numerical validity.
Compatibility can establish declared transport between values; require equality for the
fixed dependencies actually consumed rather than requiring predicted target parameters
to equal their base point.

Retain semantic points, native payloads, setups, fresh factors, sparse response actions,
tangent/secant history and verified nested roots/jets/proofs in their existing worker/pool
lifecycles. Fresh reuse policy prevents inherited native reuse. Eviction releases the actual
escaping owners; replacing/clearing one session must not silently invalidate or authorize an
independent factor.

A connected-path policy requires originating path/sheet and transport/orientation evidence.
Endpoint feasibility cannot prove connectedness. Multistart may serve an any-qualified-root
policy; it cannot rescue a connected tracker by changing sheets. The horizon's decision to
apply a predicted move remains control policy, separate from start production.

### T4 — derived families and terminal guarantees

Preparation owns enumerated derived families: parameter/arclength continuation, constructed
homotopies, artificial shifts, bounded feasibility, block subsystems, reduced space, nonlinear
preconditioners, consistent-initialization systems and surrogate correspondences.

Separate family structure from bound anchors, parameter values and pseudo-time steps.
Frame structure/value identities and coordinate maps through pse-ids; rebind stable families
without rebuilding their symbolic programs. Analyze each family's actual incidence, including
added couplings. A matching witness does not establish numerical regularity, eliminability,
independent objectives or attraction.

Each family declares reconstruction, preserved guards/bounds/inequalities/objective,
selection meaning, actual derivative support and one of:

- terminal equation identity;
- reconstruction equivalence under explicit validity conditions;
- block coverage, which does not establish simultaneous satisfaction;
- approximate correspondence with its permitted use.

Symbolica differentiates representable composition. Opaque fitting/shooting/provider oracles
supply only their admitted partial/JVP/HVP contracts. A wrapper cannot manufacture exact
derivatives. Preserve structural incidence independently of Value numerical support.

Auxiliary status, duals, stationarity and certificates do not become original evidence.
Reconstruct and assess the original at frozen final accuracy. Usually perform original
correction; omit that extra native solve only when the reconstructed terminal realization
already supplies every required original observation and permission. A finite pseudo-time
step or least-squares stationary point is not a terminal root.

### T6/T8 — scope, work and numerical accuracy

Extend ExecutionScope rather than replacing its clock. Distinguish task, durable occurrence,
mechanism slice and attempt caps. Account preparation, prediction, screening, native work,
rejected trials, proof search, factors and final assessment. Allocate one charging owner for
each consumed unit; inclusive callback work cannot be charged twice. Track actual work where
available and identify reservations/unknown counters honestly.

Task exhaustion is terminal. Local optional exhaustion is not a declaration that the base
problem is unsupported. CPU/job permits remain held through native exit and destruction;
waiting, retained proof objects and process-global native synchronization belong to the same
coordinated resource policy. Preserve finite limits and the workstation adjustment; increasing
a ceiling is not the planned evaluation-reduction mechanism.

Consumed accuracy carries output/action identities, derivative order/source, normalization
and error allowance. Derive inner residual and linear-solve tolerances from admitted error
amplification, not solely a fixed tighter factor. Root forward error depends on the local
inverse and nonlinear remainder; IFT jets also depend on derivative errors and nested products.

Distinguish certified bounds, numerical estimates and unresolved accuracy. Use verified
chart/enclosure information where it establishes a bound; otherwise combine library
conditioning/backward-error information and refinement comparisons as explicitly estimated
evidence. Each consumer declares the required evidence class. Preserve existing original
physical residual and derivative backward-error acceptance unless the operation explicitly
requires a forward/root/jet bound. Certified evidence must match scope, normalization,
demand and allowance. Estimated control can admit an operation whose contract permits it;
it cannot satisfy a certified-bound requirement. Unresolved evidence triggers bounded
refinement/canonical evaluation where that supplies the contract. Missing optional
acceleration evidence disables that mechanism and preserves a lawful base route; refuse the
task only when its required operation remains unsupported. Do not silently weaken the
demanded accuracy class.

Frozen original physical tolerances remain authoritative. Exact derivative source describes
the mathematical chain, not zero floating-point error. Accuracy promotion recomputes weaker
numerical roots/jets/factors as required while retaining unchanged structural/proof products.
No original result is qualified using an intermediate tier.

### T9/T10 — profiles, demand and traces

Build one route request from each prepared original/derived problem and its actual profile,
representation and demand. Existing pending/refusal meanings survive; a missing optional
response does not reject an otherwise lawful direct solve.

Typed profiles expose effective native controls, applicability, derivative/action demand,
setup/reuse and relevant globalization behavior. Reserve their raw option keys and reject
inert combinations. Preserve the set/unset state of complete effective configurations.
A quasi-Newton profile requests the order it actually uses rather than unused Second support.

Declare the strategy trace and required vocabulary once in pse-schema; derive Rust/Arrow/Python
transport through existing generators. Record plan versus actual route, original/derived
problem, starts/transport, phase/scope, profiles, accuracy, typed causes, permission and work.
Pre-native and report-less failures remain visible. Presentation never reconstructs policy
from metrics or strings.

Extend durable run interpretation and request/frame identities through the existing 25g
directional compatibility and explicit-migration mechanisms. Preserve historical bytes.
A strategy retries numerical work inside one occurrence; it does not create a second durable
commit/lease/retry protocol.

## Library placement and method completeness

The integration layer owns foreign details; library-owned iteration, step acceptance,
factorization and model management remain inside fitting libraries. The project owns
scientific meaning, explicit admissibility and bounded composition.

New bindings use pinned source acquisition/build through the existing native toolchain,
not ad hoc installation. Select Uno at the reviewed commit
[805a27ecf802eec33d1aea85ae73efa8a24f71be](https://github.com/cvanaret/Uno/tree/805a27ecf802eec33d1aea85ae73efa8a24f71be)
and PETSc v3.24.0 for the scoped roles below. Add egobox-ego 0.38.1 as the surrogate
management library, with its resolved family pinned consistently in Cargo. Existing POUNCE,
FERAL, faer, Symbolica and SUNDIALS pins remain their current declared versions.

| Family | Selected realization and completion boundary |
|---|---|
| M1 KKT prediction | Generalize the certified response/predictor to related targets; retain semantic screening and start-only permission. |
| M1b changing active sets | Bind pounce-sens-core release/activity/natural-unit coordinates explicitly; pounce-qp homotopy for admitted QP. Track segment coverage and correct nonlinear predictions. |
| M2 limited correctors | A bounded native-library corrector over an intermediate proposal; its partial progress never grants final permission. |
| M3 root tangent/secant | Sparse faer factor/backsolve actions at a qualified root, plus typed scaled secant history. Materialize a dense response only when requested. |
| M4 inexact Newton–Krylov | KINSOL forcing/Krylov with admitted library block factors; true demanded JVP avoids full Jacobian assembly where source support permits. |
| M5 setup/factor reuse | KINSOL setup/residual monitoring; actual retained FERAL solver state; an optional-admission Ipopt C++ ReOptimizeTNLP binding implemented for compatible sequences. |
| M6 maps/Anderson | KINSOL Picard/FP/Anderson only where settings act and map/bounds contracts fit, after truthful callback repair. |
| M7 bounded local models | Original bounded feasibility/restoration through admitted NLP solvers; Uno library filter trust-region SQP/SLP. Stationary non-root remains auxiliary. |
| M8 native globalization | Typed Ipopt profiles; POUNCE's declared second-opinion descriptions run as visible attempts under one ledger. |
| M9 continuation | Library correctors and sparse bordering with oriented tangents, localization and typed event evidence. |
| M10 constructed homotopies | Enumerated Symbolica/oracle composition over frozen original definitions, with admitted anchors/domains and terminal guarantees. |
| M11 pseudo-transient | Existing integrators for authored dynamics; PETSc or admitted shifted-root composition for explicitly derived artificial flow. |
| M12 DAE initialization | Expose IDAS consistent-initialization controls, then admitted algebraic/least-deviation preparation; preserve roles and original consistent-state assessment. |
| M13 block execution | Complete structural BTF/tear evidence, admitted library block solves, original reconstruction and assessment. |
| M14 reduced space | Existing implicit preparation with valid selected sheets, reconstructed objective/inequalities/bounds and propagated inner accuracy. |
| M15 nonlinear preconditioning | Scoped PETSc composition where fitting; otherwise shared composition over admitted library block solves, with equivalent residual/derivative action. |
| M16 surrogate/multifidelity | Retained egobox full solver/TREGO management and explicit fidelity correspondence; proposals screened/corrected against the original. |
| M17 Schur/batch/multistart | Actual POUNCE Schur use, QP paths and existing coordinated workers; record fallback and branch permission. |
| M18 evaluation reduction | Selected roots/jets and unchanged proof promotion; demanded directional programs; typed partitioned quasi-Newton/FD modes; guarded warm inner roots. |

### Foreign boundaries and scoped profiles

Uno uses library-owned filter profiles. The initial bounded SQP profile is filtersqp with
LBFGS, no inertia correction, and HiGHS QP/LP; the SLP profile is filterslp with HiGHS LP.
The library owns curvature handling/restoration. Its pinned LBFGS model is operator-only:
the scoped HiGHS binding
mechanically materializes the library-owned positive operator with checked triangular
capacity, a finite foreign allowance and original-scope polls between columns. It records
actual materialization work and owns no quasi-Newton update. Do not send an indefinite exact Hessian to
the HiGHS QP route. An exact-Hessian profile requires admitted library inertia correction
and a matching symmetric-indefinite factor binding before being advertised.

A narrow C++ bridge catches foreign exceptions from all Uno entrypoints and getters.
Rust callbacks catch panics, preserve sparse order/index width, objective/multiplier convention
and lifetimes. A positive Uno evaluation return means a recoverable trial; a negative return
is not a terminal convention. The C++ callback trampoline converts a latched terminal Rust
cause into a distinct exception outside Uno's recoverable EvaluationError family, caught by
the outer shield. Cancellation/resource/contract failures must not enter minor trust-region
retries while waiting for an iteration callback. Add the foreign sys binding through the
required crate ADR; do not expose foreign objects in workflow contracts. Uno's
[C boundary](https://github.com/cvanaret/Uno/blob/805a27ecf802eec33d1aea85ae73efa8a24f71be/interfaces/C/Uno_C_API.cpp)
and [presets](https://github.com/cvanaret/Uno/blob/805a27ecf802eec33d1aea85ae73efa8a24f71be/uno/options/Presets.cpp)
are the exact source basis. Bind matching HiGHS explicitly, configure the required C++17/
Fortran toolchain and disable uncontrolled nested OpenMP. Existing Ipopt/MUMPS linkage does
not qualify an additional Uno factor binding.

PETSc has scoped root/outer-method profiles, not ownership of process workflows.
Its tagged trust-region root method can reject invalid trials but refuses variable bounds;
an invalid initial point remains different from an invalid trial. NASM is admitted only
for unbounded, domain-safe square block composition: it does not supply general trial
backtracking or variable-bound enforcement. Domain-fragile/bounded cases use admitted
existing local solves and the shared composition with the contract gap recorded.

For admitted unbounded artificial flows select PETSc TSPSEUDO with inner SNESNEWTONTR.
Supply IFunction = M(x) * xdot + R(x) and IJacobian = dF/dx + shift * dF/dxdot with the
declared mass/sign/normalization. Register the TS candidate-domain check and distinguish
recoverable SNES domain flags from terminal error codes/latches. The library's
TSAdaptCheckStage rejects invalid domains or failed nonlinear stages and reduces the step;
TSPseudoTimeStepDefault owns growth. Declare rejection/nonlinear-failure/step/time caps and
original steady-residual termination, with a valid initial point. This replaces a project
SER controller in its admitted scope. The
[tagged stage check](https://github.com/petsc/petsc/blob/v3.24.0/src/ts/adapt/interface/tsadapt.c)
and [TSPSEUDO](https://github.com/petsc/petsc/blob/v3.24.0/src/ts/impls/pseudo/posindep.c)
supply this contract. Explicit SNES choice is necessary: the library defaults to KSPONLY,
and using nonlinear correctors is a different admitted construction from its one-step profile.
For bounded flows use shifted-root composition over an admitted bounded root/NLP solver;
the shared declared outer step control is necessary because this PETSc profile lacks bounds.

Own PETSc initialization/finalization and process-global state centrally in the native
integration. Pin real-double, serial MPIUNI and the selected integer ABI. Use per-instance
options and coordinate native concurrency with existing CPU/pool scopes. NASM supplied
sub-SNES ownership and retained scatter references have distinct destruction contracts;
bind them explicitly from the
[tagged implementation](https://github.com/petsc/petsc/blob/v3.24.0/src/snes/impls/nasm/nasm.c).
New foreign bindings carry capabilities that routing can inspect; installing them cannot
change explicit selection. Every PETSc rung names its provider and inner profile rather than
silently replacing an explicitly selected KINSOL/Ipopt method.

### Reuse, path and surrogate obligations

M1b must map primal/slack/active-row/release coordinates explicitly. The current zero-barrier
active-set KKT factor is not the library's interior-point layout. Implement release/backsolve
operations with library factors; include inequality-row breakpoint/release events.
A segment limit leaves a partial prediction, never a completely tracked path.
The pinned NLP path routine does not relinearize between breakpoints; follow it with correction.

M5 retains the numerical objects: compatible POUNCE applications reuse worker-owned FERAL
solver/backend objects rather than reinstalling a fresh factory on every call. Main,
restoration and Hessian-bypass roles have separately keyed mutable factors; factories may
request more than one object and must not lend one factor simultaneously to incompatible roles.
Reset per-attempt
statistics/options while retaining lawful symbolic analysis. KINSOL setup reuse records refresh,
staleness and monitoring; a stale iteration setup cannot become a fresh sensitivity factor.
The Ipopt C++ sequence binding retains TNLP/application state with its actual same-structure
contract; reserve the unsupported C-API spelling rather than advertising it as equivalent.

For M16 use the full egobox solver/state iteration seam, retaining TREGO phase/radius,
training, RNG and learned state within its task. The stateless suggest service does not
implement this retained model-management contract. Treat uncertainty as statistical evidence,
not a certified numerical-error bound. High-fidelity evaluations are charged to the same
task; the surrogate cannot change scientific targets or qualify itself as the original result.
The [published 0.38.1 solver source](https://docs.rs/crate/egobox-ego/0.38.1/source/src/solver/)
owns this iteration/state distinction. Its archive checksum is
ad99a4d0668707c7f7d7d605fe33e49e75cdad2ffe98421699be54ff1c9224d8;
pin compatible GP 0.36.1, MoE 0.35.3 and DoE 0.35.1 through workspace dependency/lock owners.

## Numerical preservation and evaluation reuse

### Selected-root sheets and reusable proof products

Separate competitive exclusion, winning-root existence/uniqueness, guard/order support,
sheet transport and numerical root/jet accuracy. Key each obligation on its complete
dependencies. At the same point, promote demanded order without repeating unchanged
competitive exclusion; calculate only missing local support/jets.

For nearby points, consult the validated chart before numerical proposals. A chart can
establish selected meaning only inside its admitted parameter/root neighborhood and
complete exclusion scope. Verify the new numerical root and demanded accuracy there.
Retain canonical operational Value selection whenever evidence does not establish
start-independent selected meaning. A warm root never changes selection by itself.

Derivative-worker/connected-path retention names a sheet, not only a regime. Reuse inside
one regular uniform chart supplies continuity. At a chart boundary establish a validated
bridge/common-anchor connection using the verifier. At a common parameter anchor establish
one certified root in both charts' validated uniqueness regions, or a validated bridge
explicitly linking the certified roots. A bounded uniform-chart chain covers the entire
declared parameter segment with certified adjacent connections. Endpoint certificates,
overlapping parameter/root boxes or residual-small numerical proposals alone are insufficient.
Try bounded bridge/chain construction before refusal. Failed or rejected transport leaves
the previous accepted lineage intact; commit only after verification and scope checks.
If transport remains unresolved, report a recoverable branch/continuity trial where the
outer contract permits it; do not manufacture permission from fresh endpoint certificates.

Globally selected minima may change winning sheets under a policy allowing that meaning.
Their valid endpoint derivatives do not establish a connected path. Independent cases get
independent branch state. Adaptive useful chart construction consumes one existing allowance;
larger cold-proof ceilings are not the substitute.

### Continuation and artificial flow

Use scaled tangent orientation and sparse bordering. A determinant sign change is an
indicator; classifying a simple fold requires localized rank loss, augmented regularity
and supported nondegeneracy evidence. When the demanded support is unavailable, publish
an indicator/unresolved event. A branch point is not relabelled as a fold, and temporarily
moving away from the target parameter does not prove it unreachable.

Artificial flow records row-to-state pairing, normalized units/sign, mass and frozen versus
state-dependent semantics. Default mass and anchor are frozen from the last accepted
pseudo-state throughout the next step's nonlinear trials and rejected-step retries; changing
the step size does not change that mass. Update only after acceptance. Label the flow a
heuristic when attraction is not established. Frozen-mass derivatives exclude a mass derivative.
The distinct state-dependent construction includes DM(x)[v] * xdot in the PETSc Jacobian
action, or DM(x)[v] * (x - anchor) / step in the shifted-root action, in addition to mass
and original-residual terms. Matching establishes pairing,
not attraction. Preserve the existing dynamics adapter's fixed normalized 0/1 mass scope;
general artificial mass first belongs in shifted-root preparation.

DAE initialization respects differential/algebraic roles and the library's index/IC contract.
Failure restores only scoped realizations; it never mutates the original specification.
Every resulting state is assessed against the original consistent-state requirements.

## Decision route and execution packets

Allocate real ADR identifiers when beginning the affected decision work; the IDs in front
matter identify the existing foundations, not invented acceptance of future decisions.

| Decision | Selected route and prerequisites |
|---|---|
| Declared escalation / truthful attempts | Supersede the relevant no-fallback clauses of ADR-0083/0152; preserve explicit selection and terminal classes. ADR plus scoped design review and blueprint §§18.6/18.7. |
| Derived families / identities / terminal use | ADR/design route for §§5/14/17/19 and affected hashing frames; preserve one authored authority and version changed preimages. |
| Library-owned methods versus domain composition | Governance ADR and precise PS-09 clarification through its profile/binding owner; no silent MUST waiver. |
| Starts, accuracy, trace and public interpretation | Update their contract owners under adopted decisions; generated declarations, directional compatibility and explicit preserving migration. |
| Uno/PETSc foreign crates/bindings | ADR/review, pinned build and callback/ownership capability admission before implementation claims. |
| Current defects within existing contracts | Ordinary fixes with focused controls; no unnecessary ADR. |

The previous Revise verdicts do not constitute acceptance of new ADRs. Obtain the required
scoped decision review on corrected proposals before an ADR becomes accepted; K5 is not a
substitute for that route. Architecture amendments use their owners and a blueprint revision
row. Accepted ADRs remain immutable.

Packet IDs P0–P11 are distinct from method-family IDs M1–M18. Status has one owner here.

| Packet | Prerequisites | Delivered behavior / responsible owners | Status |
|---|---|---|---|
| <a id="p0"></a>P0 Truthful current behavior | Current baseline | Native callback/status/settings repairs; composed retention permission; typed profile-chain failure preservation. Existing native/runtime/completion owners. | implemented; targeted controls passed |
| <a id="p1"></a>P1 Shared strategy contracts | P0 for consumed failure meanings; decision route for changed contracts | Declared policy, starts/products/keys, observations/history, scoped accounting, profiles and generated trace contract. Model/schema/ids/native/runtime owners together. | implemented; targeted controls passed |
| <a id="p2"></a>P2 Derived math and accuracy | P1 contracts; derived decision route | Immutable families/reconstruction, actual derivative/action support, consumed error and propagated tiers. Compiler/math plus opaque-oracle suppliers. | implemented; targeted controls passed |
| <a id="p3"></a>P3 Shared driver and direct cutover | P0/P1; P2 slice for derived execution | Pure planner/classifier/transitions, injected executor/assessment, direct minimal strategy and visible rungs. Runtime math and direct/public consumers. | implemented; targeted controls passed |
| <a id="p4"></a>P4 Selection products and continuity | P0/P1; P2 consumed-accuracy contract | Root-sheet transport, verified root/jet/proof retention/promotion and useful charts. Existing implicit/compiler/verifier/native owners. | implemented; targeted controls passed |
| <a id="p5"></a>P5 Reuse and prediction | P1/P3; P2 accuracy; P4 where nested selectors consumed | Actual setups/factors, sparse response actions and common predictor/start admission; M1/M3/M5. Native math retention and target consumers. | implemented; targeted controls passed |
| <a id="p6"></a>P6 Native profiles and globalization | P1/P3; P2 bounded-feasibility family; foreign decision route | M4/M6/M7/M8 and demanded JVP; Uno/PETSc scoped profiles and truthful visible second opinions. Native integration and compiler support. | scoped profiles implemented and tested; actual POUNCE second opinions/Hessian consumers delivered in [25n N5](25n-automatic-simulation-solve-pipeline.md#n5) |
| <a id="p7"></a>P7 Paths and homotopies | P2/P3/P5; P4 when selected functions participate | M1b/M2/M9/M10: activity binding, limited correctors, oriented tracker, events and homotopy families. Math/native/runtime path owners. | implemented; targeted controls passed |
| <a id="p8"></a>P8 Pseudo-time and DAE starts | P2/P3/P6; P4 for nested functions | M11/M12 with declared mass/flow, library solve composition and role-preserving IC recovery. Derived preparation/dynamics/native owners. | implemented; targeted controls passed |
| <a id="p9"></a>P9 Structure-driven execution | P2/P3/P4/P6; P5 response products where demanded | M13/M14/M15/M17: BTF/block/reduced/preconditioned/Schur/batch forms with original reconstruction and branch-aware multistart. Structural/math/native/runtime owners. | scoped forms implemented and tested; dependency/reconstruction corrections and actual solve Schur delivered in [25n N4/N6](25n-automatic-simulation-solve-pipeline.md#n4) |
| <a id="p10"></a>P10 Multifidelity and surrogates | P1/P2/P3; target correction from P6 | M16: fidelity correspondence and retained library model management producing screened starts. Math/runtime and library integration owner. | implemented; targeted controls passed |
| <a id="p11"></a>P11 Consumer completion and K handoff | P0–P10 functional obligations | Complete migration/deletion inventory, generated/durable consumers, registered integration fixtures and qualification handoff; no second full campaign. Runtime/public/store owners. | functional handoff complete; assembled qualification pending |

These rows are not whole-plan barriers. P4 can start once its consumed contracts exist;
it need not await every derived family or the whole driver. Native acquisition/build
preparation can proceed independently of runtime consumers after its decision route.
New trace declarations are an early P1 slice; full encoding/storage migration accompanies
each producing operation, not a deferred final mirror.

### P0 — truthful observations before retry policy

Fix KINSOL FP/Picard recoverable callback semantics, preserve a latched typed terminal cause
and distinguish step-tolerance stall from acceptable convergence. Reject settings that do
not act under the chosen method. Reserve unsupported Ipopt same-structure controls.

Replace staged assessment-only retention booleans with the composed permission plus
artifact-specific decision. Preserve block coordinate/finiteness checks where they add a
different obligation. Profile chains preserve typed numerical versus contract/resource/
operational failures; terminal causes do not generate another pin/subdivision attempt.

Accept with focused callback/status/settings tests, a composed-permission disagreement,
and numerical versus terminal profile failures. Remove the replaced status/acceptance/string
decision branches in the same change.

### P1/P2 — producer contracts before dependent policy

P1 supplies the actual strategy/start/history/accounting/profile products consumed by P3.
Generate visible vocabulary and transport from one declaration; extend identity/frame and
compatibility owners rather than writing Python mirrors. Migrate existing direct admission,
start receipts and retained keys alongside these producers.

P2 supplies derived reconstruction and consumed error/action contracts. Its first usable
slice is bounded feasibility plus one parameter family and opaque-oracle composition,
not a temporary parallel implementation. Add later enumerated families with their mechanism.
Acceptance includes lost-bound/objective controls, terminal versus auxiliary success,
an ill-conditioned root/nested chain against tighter independent values, unavailable
required accuracy, and quasi-Newton demand avoiding unused Second preparation.

Remove independently constructed route/demand/accuracy interpretations as their callers
move. Do not delete legitimate independent physical/model checks.

### P3/P4 — execution seam and expensive nested work

P3 migrates direct mathematical/modeling execution to the driver and original assessment
operation. Exercise one-rung/direct/constant/rejected execution, planned recovery, same-backend
new profile, optional refusal, terminal exhaustion, abandonment and partial traces using
scripted executors. Preserve actual native cleanup in the existing session.

P4 is the correctness/evaluation-reduction prerequisite for warm inner roots and reduction.
Use same-regime disjoint winners as the negative control and a moving regular root as the
positive control. Count actual numerical proposals and complete-exclusion work: same-point
promotion and valid-chart evaluations reuse unchanged products; changed source/domain/
criterion/accuracy invalidates precisely what it affects. Keep operational Value semantics.
Delete redundant cold evaluation/proof paths once target coverage exists.

### P5–P10 — integrate mechanisms with their consumers

P5 migrates study/path/horizon prediction through one predictor; horizon application remains
its owner. Prove actual setup/symbolic-object reuse and changed-pattern invalidation.
Use fresh accepted-point factors for response, stale setups only for admitted iteration.

P6 supplies typed profiles, callback contracts and foreign bindings. Exercise bounded Uno
SQP/SLP, recoverable versus terminal foreign errors, inert-option refusal, actual effective
settings and native-free mechanism admission. PETSc root controls distinguish invalid initial
points from recoverable invalid trials and refuse variable bounds. A newly available backend does not alter an
explicit selection. Record requested Schur/globalization actually acting, not only requested.

P7 keeps natural parameter and connected arclength paths distinct. Exercise active-set
activation/release and partial segment limits; simple-fold and branch-point controls;
recoverable-domain trials; and connected-path refusal of multistart replacement. Homotopy
terminal work reconstructs and assesses the original.

P8 exercises authored versus artificial time, matched but unstable heuristic flow,
frozen/state-dependent mass derivatives, IC role preservation and original consistent-state
checks. PETSc candidate-domain rejection reduces the timestep, while terminal latches and
rejection/nonlinear-failure caps stop; rejected retries preserve the frozen mass and anchor.
Remove old per-workflow pseudo-time/recovery control when replaced.

P9 migrates conditional blocks/recycles and eligible whole-case execution forms. Exercise
coupled objective/inequality reconstruction, failed-block trial behavior, nonempty original
incidence under Value support, true JVP versus assembled reference, actual Schur use/fallback,
and coordinated branch-aware multistart. Preserve bounds by reconstruction/admission,
not by pretending a sign-constrained solver supplies arbitrary intervals.
NASM controls cover bounds refusal, invalid aggregate-update classification and
sub-SNES/scatter destruction ownership.

P10 exercises fidelity mismatch/refusal, retained TREGO state, interruption, original
high-fidelity correction and truthful statistical uncertainty. Delete project model-management
duplication where the library supplies it.

Migration happens in each producing packet. P11 completes the inventory; it is not permission
to retain old policy/callers until then. No new mechanism gets only a synthetic executor:
its packet also includes a bounded actual library contract control.

### Consumer ownership and deletion

| Consumer | Shared capability adopted | Policy retained by that consumer |
|---|---|---|
| Direct/modeling case | Preparation, starts, profiles, driver and original permission | Selected analysis/model checks |
| Staged/block initialization and recycle | Derived/block execution, numerical transitions and retention | Authored stage/tear intent and overlays |
| Natural/homotopy paths | Common predictor/corrector, branch and subdivision operations | Authored endpoints/path permission |
| Studies | Strategy per bound target/occurrence; typed seed decisions | Dependencies, samples, durable effects/leases/idempotency |
| Fitting/profile likelihood | Oracle adapter, strategy and typed solve failures | Statistical objective, confidence threshold/bracketing and sample policy |
| Horizons | Prediction/correction/retention | Control application and horizon/sample updates |
| Shooting | Opaque oracle support, routing and common completion | Endpoint/boundary targets and integrator configuration |
| Dynamics/DAE start | Consistent initialization and shared failure/accuracy contracts | Authored time/event semantics and library time-step policy |
| Python/direct/publication | Generated requests/results and actual strategy trace | Mechanical projection; no reconstructed numerical policy |

Delete replaced private numerical retry/subdivision, start transport and failure classifiers,
copied request constructors and independent retention predicates after callers migrate and
targeted checks pass. Do not delete a statistical bracket, durable transition or integrator
loop merely because it is adaptive. No compatibility shim, feature-flagged legacy path or
deleted-mechanism test remains. Existing independent oracles/historical-format fixtures
remain only for continuing contracts.

## Finding dispositions

Both source reviews remain unchanged. IDs in this section are qualified by their review;
they do not collide with the earlier Plan 25 reviews. The dispositions below reconcile the
supported functional obligations with present source and executed controls on 2026-10-05.
Resolved here means the finding's functional correction is implemented and its named controls
passed; it does not close this plan or Plan 25k K3/K4/K5. The current evidence boundary is
[the functional reconciliation](#functional-reconciliation-2026-10-05); full scientific campaigns,
required dev-profile measurements and final acceptance remain with 25k. Complete methods are
also covered by the family table above, with the subsequent automatic consumers owned by 25n.

| Earlier-review finding | Scenario reference | Disposition | Decision/work owner | Closure evidence |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f01) callback recovery semantics | [S07](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s07) | resolved | P0 | Tested: `native_failed_fp_and_picard_do_not_consume_previous_outputs`, `native_newton_line_search_recovers_domain_trial` and `recovered_trials_do_not_poison_success_but_panics_do`; failed buffers do not publish, terminal map failures latch, recoverable trial causes clear at native completion. |
| [F02](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f02) outcome/abandonment observations | [S06](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s06) | resolved | P0/P1/P3 | Tested: `callback_abandonment_discards_output_and_counts_owned_work_once`, `limited_without_stagnation_does_not_invent_failure_or_recovery` and `late_native_batch_member_keeps_actual_work_and_cannot_grant_original_permission`; attempt observations remain separate from original permission. |
| [F03](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f03) inert controls | [S11](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s11) | resolved | P0/P6 | Tested: `method_controls_must_act_under_the_selected_strategy`, `scoped_profile_refuses_required_unconsumed_controls` and `compiler_first_order_partitioned_and_fd_profiles_act_and_assess_original`; inert/unsupported controls refuse and actual settings act. |
| [F04](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f04) acceptance/retention | [S04](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s04) | resolved | P0/P1/P3 | Tested: `session_retention_requires_composed_permission_and_artifact_admission` and `all_fixed_publication_keeps_constant_values_without_kkt_evidence`; composed permission and retained components keep separate obligations. |
| [F05](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f05) documentation meaning | [S04](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s04) | resolved | Decision route/P1/P11 | Implemented: blueprint §§17.6/18.7 and revision 111 describe actual staged starts, study-owned complete native seed transport and composed permission plus session-artifact admission; bounded source reconciliation accepted scoped, 2026-10-04; accepted ADRs unchanged |
| [F06](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f06) numerical strategy ownership | [S10](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s10) | resolved | P1/P3/P7–P11 | Tested: `failed_trajectory_can_run_declared_same_backend_profile_and_only_original_permission_finishes`, `terminal_profile_causes_never_subdivide_and_keep_the_original_error` and `later_horizon_failure_retains_native_prefix_and_attributes_only_actual_failed_target`; shared numerics preserve scientific and durable policy. |
| [F07](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f07) route construction | [S11](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s11) | resolved | P1/P3/P11 | Tested: `failed_trajectory_can_run_declared_same_backend_profile_and_only_original_permission_finishes` and `callable_runs_actual_same_backend_profile_after_recoverable_observation`; a failed trajectory does not poison backend capability. |
| [F08](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f08) executor isolation | [S12](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s12) | resolved | P3/P11 | Tested: `native_success_and_auxiliary_success_cannot_bypass_original_permission` and `optional_component_numerical_failure_retains_observation_and_charges_before_direct`; injected executor/assessment controls exercise the production shared driver. |
| [F09](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f09) starts | [S05](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s05) | resolved | P1/P5/P11 | Tested: `entry_start_precedence_excludes_inherited_and_explicit_substitution`, `connected_incomplete_and_nonfinite_seeds_are_refused_before_native_work` and `shared_original_driver_refuses_fresh_seed_without_outer_recovery_grant`; explicit/no-prior entry and later grants remain distinct. |
| [F10](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f10) derived systems | [S06](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s06) | resolved | P2/P7–P10 | Tested: `anchored_homotopy_freezes_actual_anchor_residual_and_only_exact_terminal_is_original`, `actual_single_root_full_reconstruction_preserves_native_factory_and_original_assessment` and `actual_shifted_frozen_mass_retains_offset_and_submission_scope`; derived/native outcomes cannot substitute for original assessment. |
| [F11](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f11) prediction keys | [S01](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s01) | resolved | P1/P5 | Tested: `actual_producer_state_consumes_exact_point_source_order_normalization_and_class`, `reduced_product_scope_includes_actual_point_direction_source_validity_sheet_and_outer_realization` and `native_layout_and_effective_factor_profile_invalidate_retained_backends_and_require_reuse_refuses`; semantic products and native state have their actual distinct keys. |
| [F12](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f12) reuse leverage | [S02](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s02) | resolved | P4/P5/P7 | Tested: `compatible_native_setup_is_actually_reused_and_fresh_policy_refreshes_it`, `compatible_applications_retain_actual_feral_backend_and_observe_attempt_factor_deltas` and `retained_chart_promotes_guard_order_without_repeating_competitive_covering`; actual object/proof reuse and invalidation are established functionally. Required K4 distributions and retained-resource measurements remain pending. |
| [F13](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f13) budget/accuracy/observations | [S08](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s08) | resolved | P1/P2/P3 | Tested: `actual_work_is_charged_once_unknown_is_not_zero_and_task_cap_is_terminal`, `strict_unknown_inclusive_work_refuses_before_dispatch_and_keeps_complete_reservation_unknown` and `implicit_actual_native_deadline_survives_provider_and_outer_callback_boundary`; actual shared scope/work and consumed accuracy remain authoritative. |
| [F14](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f14) profiles | [S06](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s06) | resolved | P1/P6 | Tested: `actual_auto_pounce_stationary_failure_runs_generated_second_opinion` plus actual Schur and `compiler_first_order_partitioned_and_fd_profiles_act_and_assess_original` controls. The 25n N5/N6 consumers now act with visible profiles; no convergence or speed claim follows. |
| [F15](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f15) structure shapes execution | [S07](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s07) | resolved | P2/P9 | Tested: `automatic_blocks_execute_complete_nonport_coupled_original_and_keep_actual_components`, `automatic_blocks_merge_domain_control_cycle_and_refuse_single_block`, `actual_schur_unsuitable_separator_and_failed_ff_use_monolithic_disjointly` and causal reconstruction controls; original coupling/qualification survive the 25n N4/N6 corrections. |
| [F16](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f16) costly JVP/preconditioning | [S07](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s07) | resolved | P2/P6/P9 | Tested: `directional_only_source_solves_without_claiming_an_assembled_jacobian`, `demanded_krylov_route_prepares_actions_from_value_under_a_narrow_jet_budget` and `actual_kinsol_block_factors_solve_coupled_original_and_rebuild_changed_sources`; actual directional support and library factors act without fabricated assembly. |
| [F17](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#f17) inner warm roots | [S08](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s08) | resolved | P4/P9 | Tested: `selected_sheet_moves_with_uniform_chart_and_certified_common_anchor`, `disjoint_winners_in_one_regime_do_not_establish_selected_sheet_continuity` and `reduced_compiler_source_refines_consumed_accuracy_then_original_corrector_qualifies`; warmth alone establishes neither sheet nor accuracy. |

| Integrated-review finding | Scenario reference | Disposition | Decision/work owner | Closure evidence |
|---|---|---|---|---|
| [IP01](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip01) completion/retention | [S12](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s12) | resolved | P0/P1/P3 | Tested: `session_retention_requires_composed_permission_and_artifact_admission`, `all_fixed_publication_keeps_constant_values_without_kkt_evidence` and `repeated_completed_blocks_fit_short_pool_and_extracted_components_hold_grant`; original permission, component reports and escaping retention remain separate. |
| [IP02](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip02) eligibility/history | [S11](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s11) | resolved | P1/P3 | Tested: `failed_trajectory_can_run_declared_same_backend_profile_and_only_original_permission_finishes`, `callable_runs_actual_same_backend_profile_after_recoverable_observation` and `native_layout_and_effective_factor_profile_invalidate_retained_backends_and_require_reuse_refuses`; history does not overwrite eligibility. |
| [IP03](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip03) scoped work | [S12](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s12) | resolved | P1/P3/P11 | Tested: `optional_slice_refusal_keeps_base_admitted_and_charges_only_actual_work`, `failed_dispatched_effect_is_charged_once_and_unknown_work_stays_unknown` and `shared_original_driver_coordinates_fresh_seeds_and_stops_after_permission`; each actual original-assessment occurrence has a distinct charge owner. |
| [IP04](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip04) starts/connectedness | [S05](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s05) | resolved | P1/P4/P5/P7 | Tested: `preparation_first_uses_entry_then_auxiliary_correction_requires_declared_recovery`, `authored_bounds_refuse_fresh_seed_without_clamping_or_source_mutation` and `genuine_compiled_sheet_binds_prior_origin_then_certifies_actual_target_and_rejects_root_jump`; screened starts preserve authored bounds and connectedness. |
| [IP05](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip05) terminal evidence | [S06](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s06) | resolved | P2/P3/P7–P10 | Tested: `effective_output_obligations_precede_permission_and_preserve_dispatched_work`, `terminal_typed_observations_cannot_escape_as_proposals` and `actual_single_root_full_reconstruction_preserves_native_factory_and_original_assessment`; original obligations, not auxiliary status, authorize use. |
| [IP06](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip06) scientific policy | [S03](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s03) | resolved | P3/P7–P11 | Tested: `scientific_fit_strict_composed_work_refuses_before_evaluation`, `scientific_shooting_strict_composed_work_refuses_before_integration`, profile-chain controls and `later_horizon_failure_retains_native_prefix_and_attributes_only_actual_failed_target`; scientific/control/occurrence policy remains with its consumer. |
| [IP07](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip07) profile failures | [S03](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s03) | resolved | P0/P3/P11 | Tested: `terminal_profile_causes_never_subdivide_and_keep_the_original_error`, `nested_provider_trial_can_subdivide_but_native_terminal_latch_cannot` and `numerical_failure_subdivides_then_brackets_without_replacing_statistical_policy`; true causes govern lawful subdivision. |
| [IP08](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip08) root sheets | [S08](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s08) | resolved | P4/P7/P9 | Tested: `disjoint_winners_in_one_regime_do_not_establish_selected_sheet_continuity`, `selected_sheet_moves_with_uniform_chart_and_certified_common_anchor` and `actual_ibex_sheet_transport_consumes_original_endpoints_and_coverage`; selected-function coverage is stronger than an overlapping root chain. |
| [IP09](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip09) propagated accuracy | [S08](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s08) | resolved | P2/P4/P9 | Tested: `composite_uniform_supplier_consumes_and_amplifies_actual_predecessor_error`, `reduced_validity_and_consumed_forward_accuracy_refuse_without_manufacturing_certificates` and `reduced_compiler_source_refines_consumed_accuracy_then_original_corrector_qualifies`; source/class/error requirements are consumed. |
| [IP10](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip10) folds/mass | [S02](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s02) | resolved | P7/P8 | Tested: `actual_compiled_second_curvature_localizes_fold_and_first_only_keeps_it_unresolved`, `actual_compiled_branch_singularity_is_unresolved_despite_second_curvature_and_source_mismatch_refuses`, `state_dependent_mass_uses_actual_dm_action_and_matches_independent_derivative` and PETSc `trust_owner_preserves_domain_phase_causes_limits_and_terminal_lifetime` (invokes `exercise_declared_flow_and_blocks`: actual state-dependent mass/DM success and `exercise_unstable_flow`: positive native work with non-Success/None/Unqualified/infeasible original result); scoped path/mass evidence retains unresolved cases and auxiliary nonpermission. |
| [IP11](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip11) actual reuse | [S08](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s08) | resolved | P4/P5 | Tested: `retained_actual_feral_solver_reuses_symbolic_factor_and_resets_attempt_counts`, `changed_actual_pattern_refreshes_structure_without_using_previous_numeric_factor` and `compatible_native_setup_is_actually_reused_and_fresh_policy_refreshes_it`; actual native/proof ownership and invalidation are functional corrections. Required K4 measurements remain pending. |
| [IP12](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md#ip12) library guidance | [S11](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s11) | resolved | P6/P7/P8/P9 | Tested: actual Uno/PETSc scoped callback/lifetime controls, `original_bound_release_uses_actual_refactor_and_natural_units`, `public_ic_preserves_original_roles_and_has_no_physical_trajectory` and the real Auto POUNCE second-opinion control. Tagged/coordinate bindings remain scoped to their acted library routes. |

| [K5-F05](../design_review/reviews/design_review_assembled-integrated-solve-pipeline_2026-10-03.md#k5-f05--composition-indirection-omitted-escaping-box-bodies) boxed composition payload admission | [S08](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s08) | resolved | P11 / runtime math composition owner | Tested: `composition_retains_boxed_rung_bodies_and_refuses_a_short_pool` in the final 21-control runtime batch (force-validation, KINSOL/Ipopt/PETSc/Uno, 2026-10-04); checked extent, shared escaping lifetime/release and short-pool refusal |

These rows reference the earlier review's S01–S12 and the integrated review's stated closure
cases; unchanged definitions are not copied into a new scenario registry. A finding becomes
resolved only when all contributing obligations and named evidence exist. Authoring a plan
or accepting an ADR resolves none of these implementation findings.

## Verification and qualification handoff

**Interface-checked:** the two reviews, their pinned
[library refresh](../design_review/evidence/integrated-solve-pipeline-2026-10-03/library-contract-refresh.md),
and selective current-source foundation assessment. **Proposed:** this combined design,
new bindings, work packages and expected reduction in expensive work. No new product
execution, performance measurement or solver qualification is claimed by plan authoring.

During functional implementation use recipe-owned compilation and focused unit/native
controls with explicit pse-relations/force-validate. Native checks use the existing memory
cap and selected feature graph. Each new mechanism has its relevant actual binding test;
scripted policy tests do not qualify a foreign library. Run codegen when declarations change.
Delete replaced mechanisms/callers/tests immediately after migration and targeted acceptance.

P11 supplies registered fixtures and a current handoff, then K owns the single final
format/hygiene/static, integrated/native/Python, compatibility, resource/reuse and architectural
campaign. Use the current just --list contract; existing routes include check-package,
unit-package, unit-native-package, unit-native-selected and check-native-contracts.
No packet runs a full campaign under a unit label.

K3 includes the original nested PR and CSTR cases and independent expectations with their
authored bounds, inputs and final tolerances. Prior timeouts/inconclusive attempts remain
unsuccessful receipts. New strategies do not qualify a changed fixture as the old case.
K also exercises studies, fitting, horizon, initialization/recycle, shooting, dynamic starts
and durable interruption through actual assembled consumers.

K4 retains the maintainer's current dev-profile choice. Add actual proof/evaluation/setup/
factor counts, preparation/rebinding, cold/warm behavior and escaping ownership to its
existing measurement owners. Quantitative speed claims require comparable end-to-end
conditions; method inclusion has no invented speedup threshold. Keep resource-lifetime
and required reuse evidence distinct from optional extra timing comparisons.

K5 reviews the assembled target under the selected standard and adopted decision changes.
The source reviews do not establish the resulting architecture's fitness. After qualification,
complete the already authorized worktree assessment/removal at its K owner; neither plan
authoring nor differences alone justify retaining obsolete worktrees.

### Completion conditions

- P0–P11 supply all target contracts, applicable method integrations and migrated consumers.
- Required decision reviews/ADRs, architecture owners, generated boundaries and preserving
  durable transitions are complete through their sanctioned routes.
- Every adopted finding has its full correction/evidence, or a reviewed changed conclusion
  with an explicit owner; no silently omitted method or unowned decision remains.
- Plan 25k establishes its named integrated, reuse/resource and architecture evidence
  against the zero target, with actual limits recorded.
- Enduring meaning moves to its architecture/rationale owners before completed plans/reviews
  retire through ADR-0096.

### Execution checkpoint

The dated entries below retain their historical conditions. The current functional state is
[the 2026-10-05 reconciliation](#functional-reconciliation-2026-10-05); earlier outstanding-work
statements are not a second current backlog.

The maintainer committed and pushed the implementation as
`58b700bc36d59517d77886496c6ad4072ed4a560`. The subsequent
[automatic simulation/solve pipeline review](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md)
assesses that baseline, identifies additional integration corrections and distinguishes
them from the broader automatic-composition target. The subsequent
[25n](25n-automatic-simulation-solve-pipeline.md) owns AF-01–AF-08 and the corrections.
The P6/P9 packet rows distinguish delivered scoped forms from missing native obligations now assigned to
25n N5/N6; earlier finding ownership and receipts stay here. The 25n execution checkpoint and
Outcome own its current correction evidence and handoff; 25k owns full qualification.

The shared driver now consumes actual prepared original, derived, arclength and PETSc
operations under one original task scope. Only an original correction followed by the
injected original assessment can finish with result permission. Auxiliary phases produce
screened starts and separate source/derivation/transport observations. Their native work
is charged by the actual operation, including failed calls and unknown counters.
Entry-start precedence and later recovery permission remain distinct.

Bounded feasibility, anchored homotopy, reduced reconstruction and least-deviation
initialization use actual compiled physical inventories and preserve original obligations.
Initialization routing distinguishes square root initialization from an initialization NLP
using its objective and constraint facts. The runtime path producer projects the complete
original First program and exact residual offsets; its chart producer preserves the native
state-then-parameter coordinate inventory. Connected completion retains actual chart
coverage and checks the final original corrector against the ending sheet. Proof workspace
is charged separately within the finite deployment workspace and shared pool.

Sparse root prediction and retained TREGO proposals have actual original screening and
correction journeys. IDAS consistent initialization binds authored roles and initial time;
PETSc binds complete compiled BTF blocks and original guards. These targeted controls do
not establish assembled qualification. Scientific shooting/fitting submissions now retain
the deadline stamped before queueing; numerical failure classification is shared while
statistical brackets and authored workflow policy retain their owners.

The native owners retain actual KINSOL/FERAL setups, Ipopt application/TNLP sequences,
HiGHS/POUNCE activity products, Uno globalization and PETSc local solver state. Directional
compiler programs and reduced residual realization use actual demanded support rather than
promoting First to Second. Replaced native Ipopt execution and private failure classifiers
are removed. ADR-0154–0157 remain proposed; the prior proposal review does not close K5.

Actual prepared multistart, retained surrogate phases and opaque fitting/shooting operations
now use the shared driver. Screening retains partial callback counts on failure, and local
optional derived deadlines are distinguished from task exhaustion. Native numerical limits
do not imply stagnation or authorize homotopy subdivision. Numerical path events retain
actual localization, rank, nondegeneracy, work and optional genuine compiler Second support;
First preparation alone cannot establish curvature. Trace clones share one retained owner.

The generated numerical declaration is exposed through the prepared public operation;
Python composition delegates validation and execution to Rust. Generated event projection
preserves auxiliary observations separately from original result permission, including the
public run-result projection. Focused path-event, optional-deadline and public Python controls
pass. Absolute deadline expiry now retains its typed scope cause through mathematical and
provider boundaries rather than being classified from a resource-limit message.

Demand-driven point/JVP refinement now consumes the actual compiled source and preserves
typed local refusal, source/validity identity and the original finite task scope. Optional
preparation may decline only under its declared recovery; a required product or task-level
resource failure stops. Preparation waits, structural tear work and every conditional
initialization block retain the deadline stamped at submission. Standalone cone, recycle
and initialization reports project their retained traces with the submitted run identity.
Measurement hooks observe actual prepared support, native work and retained owners, keeping
unknown counters distinct from zero and inclusive observations separate from ledger charges.

Functional handoff, 2026-10-04: P0–P10 and P11 migration/deletion obligations are
implemented. The bounded assembled review accepts the inspected architecture and its scoped
functional evidence. The final M4 controls establish library block-factor preconditioning,
actual compiled First/JVP preparation for main and conditional routes, and full-system
coupling. M6 and every Krylov route admit combined finite workspace before native allocation.
Public Python controls decode the actual Arrow strategy events with stable submitted run
identities. Registry generation and compiled-API stubs are current. The pinned PETSc capacity
patch is reproducibly applied, its installed-header capacity control passes, and the fresh
native prefix passes the tagged-error, callback and lifetime controls.

Next: Plan 25k owns the single formatting/static, assembled behavioral and scientific,
dev-profile reuse/resource and final acceptance campaign. These targeted controls do not
qualify unexercised integrated scope. Historical worktree cleanup follows successful
qualification and the existing semantic assessment.

Assembled qualification repair, 2026-10-04: the first full default/native campaign exposed
missing mandatory preparation dependencies, a missing append-only native-backend schema
transition, representation-specific evidence admission and native continuation seams.
The campaign remains a diagnostic inventory; source changed during repairs and its earlier
passes do not qualify the repaired tree. Repairs preserve original fixtures and scientific
criteria, demand-driven preparation, immutable migration history and typed causes. A finite
compiler-owned class-proof allowance replaces an unconfigurable internal ceiling, without
resetting consumed work or enlarging byte reservations. Complete qualification on a stable
tree, dev-profile measurement and worktree retirement remain outstanding in Plan 25k.

The remaining campaign seams have explicit owners: compiler assembly allowances travel
with the complete preparation profile and through upgrades; its view identity uses a new
v4 frame while evaluator artifact keys retain their consumed policy. Diffsol's library-owned
configuration is serialized in every build, independently of native-driver availability.
Fixed-assignment SCIP refinement prepares genuine separately demanded callbacks while the
primary factorable program retains Value order. Scientific campaigns declare larger finite
allowances without changing physical inputs or acceptance criteria. The next step is to
verify these seams, freeze the assembled source, and consume K3/K4/K5 evidence.

The policy projections now preserve all three assembly controls through functions,
conditional blocks and parametric preparation. Targeted checks establish exact-cap
admission, one-byte-short refusal, complete transport and genuine fixed-assignment
callbacks. The original PC-SAFT scientific control and affected public transient fitting
journey now pass with their explicit policy. The bounded repair review has no remaining
material source findings. Freeze this assembled revision for the complete local gate,
then run the seed campaign and selected dev measurements before closing the plans or
removing semantically obsolete worktrees.

The first stable attempt exposed the omitted approved v4 catalog addition; the historical
frame vectors remain unchanged. Focused durable review also identified that SQL readiness
fingerprints do not version opaque JSON profiles. Current study operation/request, study
definition and job payload envelopes now have explicit new versions, with unsupported
historical readmission refused before decoding changed nested inputs. Stored bytes and
histories remain intact. Regenerate and verify these boundaries before restarting the
stable qualification campaign; the interrupted attempt remains diagnostic evidence.

The version controls now pass current roundtrips, incomplete historical readmission and
exact stored-payload preservation. Public durable publication and cancellation also pass
after migrating their complete fixture policy. Historical identity vectors retain their
original bytes and the approved v4 addition is explicit. Restart the complete local gate
on this corrected revision, then consume the seed, measurement and bounded closure evidence.

The final native campaign exposed three stale qualification consumers: the PR derivative
control's superseded manifest fields, worker budgets below the compiler's actual declared
scratch requirement, and a limited-incumbent assertion that conflated native candidate kind
with original feasibility permission. Their repairs consume the complete policy, admit
the unchanged scientific operation with finite capacity, and assert actual fixed-assignment
provenance alongside original feasibility and nonoptimality. The worker CLI default now
admits ordinary default compilation through its unchanged resource mapping; explicit smaller
deployments still refuse. Targeted verification precedes the final frozen campaign.

The original thousand-point flash study exposed a numerical-policy propagation gap:
state reconstruction equations carry authored physical tolerances, but those tolerances
were omitted from solver row requirements. Source-issued active-row requirements now
survive subsequent preparation and native accuracy resolution, with idempotent replacement
under their producer-owned identities. Focused controls exercise actual prepared-solver
accuracy, canonical unit conversion and unchanged original quality; supplied states retain
their independent checks without phantom row budgets. The measurement consumers also now declare the seed fixture's
existing property permission and enter the executor while submitting supervised warm-up
work. Verify these boundaries before freezing the final qualification and measurement tree.

The bounded closure assessment added actual compiled ill-conditioned nested point/JVP
refinement against a separately tighter numerical reference, and an unstable artificial-flow
control whose auxiliary trajectory cannot grant original permission. Both controls pass;
the review accepts their specific evidence boundaries. Compiler-issued selector evidence
requires authored finite bounds before isolation; caller solve boxes do not create that
authority. The start/retention owner now describes the implemented staged variants,
study-owned complete native seeds and separate session-artifact admission. The unchanged
thousand-point flash control now passes native success, original physical checks and
structure reuse at every point. Sampled reporting stacks identified repeated immutable
source projection/validation; share its checked, allocation-owned product and remove
redundant single-source merging before freezing the current assembled campaign.

The complete seed inventory has now supplied the remaining scientific repair targets.
Coefficient extraction's global-box obligations were incorrectly inherited by callback
representations; callback admission now consumes its own guards, original contract,
structure and derivatives, while genuinely admissible higher coefficient candidates still
retain unresolved evidence and ordered priority. Authored repairs remove duplicate ideal-gas
normalization and supply narrow fixture permissions for the selected predictive rule and
the critical-endpoint form-negative test. The reaction-law fixture now instantiates its
parameter-only law while retaining static definition-owned structural selection. Remaining
work is the checked export metadata lifetime, fixed-input derivative demand, nested
neighborhood preparation and observed recycle-failure attribution, followed by current
scientific verification and the frozen campaign; the earlier native seed run remains
diagnostic evidence.


The checked source-export optimization now shares immutable checked storage and admits
storage/map metadata before allocation. Escaping checked tables retain those grants after
package and result drops; raw Arrow candidate extraction has a separate receiving-admission
boundary and does not claim checked metadata ownership. The bounded review accepts that
scope, and foundation and runtime lifetime controls pass. The original thousand-point study
completed its scientific checks, but concurrent source edits invalidated its source-stability
guard; a frozen-tree rerun remains required. Accelerated density definitions declare a mathematical branch over
their original density interval, independently of operational KINSOL selection; the bounded
constructor has an actual oracle fixture. Fixed-input provider demands must follow the bound
case's derivative coordinates, while genuinely varying nested inputs retain their required
order. Current repairs also retain typed failing body/output/coordinate attribution rather
than promoting unsupported selector capabilities. The overheated recycle failure has reached
its original trial-rejection boundary; its exact observed lineage and unchanged-specification
recovery still need verification. These are qualification repairs, not closure evidence.

The next repair boundary is explicit physical accuracy and lawful selected-function preparation.
Active conservation tolerances now reach native preparation with canonical units, provenance,
override precedence and idempotent retained inputs. Expected-failure connection members lower
to their exact generated rows, including indexed occurrences and stage-stable identity;
the strict lineage matcher is unchanged. Nested ideal projection retains branch-local guard
attribution when an identical guard is already enforced unconditionally at the required order.
Uncovered guards still refuse. The native verifier adds library-owned unknown-coordinate
shaving to its budget-checked contraction pipeline and changes verifier identity accordingly.

The recycle's separated liquid now retains its phase through the receiving caloric state,
using the original property data and a shared mixture-enthalpy expression; the original
transport checks and oracle expectations remain. The heat-exchanger diagnostic distinguishes
the scaled density polynomial from the required physical pressure residual. Relational density
therefore reconstructs the pressure-defined state with an explicit physical pressure budget,
while accelerated polynomial selection remains separate. Original pressure-case convergence,
nested scientific checks and specification-restoration verification are still underway.
The PR and smooth-flash diagnostic traces used a declared direct-only strategy; no recovery
profile was silently withheld or added. Full qualification, the dev measurements and cleanup
remain with 25k after these source repairs are stable.

Current scientific repairs retain exact authored domains and independent accuracy budgets.
Selected-function projection admits guarded exact rational powers through supported library
operations; unguarded dyadic roots preserve their zero and pole behavior, while unsupported
general unguarded powers still refuse. Package checking enumerates actual qualified-name
prefixes and retains the original resolver's visibility and shadowing authority. Ipopt now
combines its independently enforced convergence components without imposing the tightest
physical row budget on unrelated KKT errors; POUNCE's different stopping contract remains
unchanged. Focused projection, name-resolution, linked native and prepared accuracy controls
pass. Their current original-case campaigns remain qualification work, not established
whole-case results.

The ordinary phase-preserving recycle and its overheated failure/restoration control now
pass their original scientific obligations. The four-feed study still needs a qualified tear
start: its retained trial lineage identifies the cold liquid receiver, but does not retain
the rejected trial temperature. An in-memory initializer using the existing heater target
is being assessed with unchanged final physics, feed values and positive/negative checks.
The interval verifier's remaining native contraction cost is being addressed through guarded
library split selection, preserving covering, parameter exclusion and stopping precision.

Guarded derivative-based interval splitting now passes the linked adapter controls without
changing covering or precision. Failed derivative reports retain their actual typed callback
cause. Finite derivative and Jacobian diagnostics stamp their scope before preparation and
carry it through queued worker admission, nested factories and native execution; late work
cannot renew that deadline. The actual nested-provider and queued-preparation controls pass.
The refreshed nested scientific campaign consumes these repairs. The original pressure
cases are physically feasible but still iteration-limited, so native result permission is
not granted. Their stopping metrics require assessment before further numerical changes.

The capped PR diagnostic satisfies every original physical check but stalls with large
bound multipliers on points interior to the authored boxes. The local PR tangent-plane
probe likewise satisfies its physical reference and TPD oracle while recovered original
complementarity fails. The pinned presolve binding lacks general tightened-bound dual
provenance. Its native-NLP admission must exclude those unsupported transformations rather
than weaken original KKT acceptance or substitute bounds after row removal; supported
affine elimination and row/rank analysis retain their distinct contracts. Targeted native
geometry and dual-attribution controls precede another current scientific check.

The presolve boundary now separates executable transformations from source proof analysis.
Linked controls retain authored barrier geometry, required rows, original affine multiplier
ownership and the analytic singleton sensitivity under both Off and Auto. Required untracked
bound transformations refuse before dispatch. The local PR liquid tangent-plane probe now
qualifies stationarity and original complementarity with unchanged scientific criteria;
its distinct current reference fixture replaces the old defect-dependent campaign failure
expectation. Dedicated SCIP reproducer/contradiction controls remain and R-52 stays open.
The current pressure diagnostic likewise passes original KKT checks, but still ends at its
explicit iteration cap in restoration; that diagnostic does not establish native success.
Current disk-fixture and original-budget campaigns precede the frozen assembled campaign.

The remaining pressure stopping mismatch is now corrected at the native coordinate boundary:
each original physical row budget determines its consumed row scale relative to the resolved
feasibility threshold. Callback derivatives, multipliers, warm identities and original recovery
use that map coherently; declared nominals remain the original KKT authority. Both maps are
attempt-owned, with predecessor maps released before Auto fallback rebuilds. Licensed math
and linked native controls pass, and bounded independent review accepts the ownership and
transport. Selected-sheet continuation now tries the previous root enclosure and then a hull
of independently certified endpoints as Newton proposals under the same proof account;
all original guard, domain, covering and bridge obligations remain. Specific chart refusal
reasons survive. Current original-step ideal and original-budget pressure cases precede the
frozen full campaign; nested PR deadlines remain unresolved.

The original pressure quartet now passes native success, original scientific checks and
the original derivative diagnostics after the row-coordinate repair. The two pure ideal
selected functions still refuse their original pressure perturbation: a captured native
chain exhausted interval bisection while the root stayed nearly fixed, so its endpoint
existence hull remained too narrow. The next proposal uses the certified uniqueness hull
without deriving permission from it. The competitive contractor also selects the pinned
library's outer Taylor relaxation to avoid repeated Hansen Jacobian evaluations; complete
covering and all original guards remain mandatory. Bounded source review accepts these
changes and existing native proof controls pass. Their original scientific cases still
need current verification before the final assembled campaign.

The three original nested ideal liquid/two-phase/vapor fixtures now pass their
original derivative perturbation and every evaluated scientific obligation. The pressure-scaled
mixed-root native control passes with actual endpoint certification and connected-chart work.
The two original nested PR cases are still running under their original finite budgets. All
functional repairs are frozen for formatting/static checks; their result will determine whether
the complete authored and assembled campaigns can proceed without further source repairs.

The PR proof-cost repair now addresses useful certificate construction: the old parameter
neighborhood was narrower than the original diagnostic perturbation, requiring another
complete competitive covering at sampled points. Broader finite parameter/local-root proposals
precede the retained narrow fallbacks, with unchanged original domains, score/tolerance,
uniform Newton, guards, all-regime covering and total proof account. Nearby-point refinement
and a competing-switch negative control exercise actual certificate containment and reuse.
The superseded verifier campaign was deliberately stopped without claiming qualification;
one current original PR case will assess this repair before the full campaign is repeated.

The wider scalar proposal still fell back to a narrow chart at an original PR feed
perturbation and repeated complete competitive covering. Both original-budget runs ended
inconclusive at their provider deadlines. Replaying the actual captured request identified
an interval-inverse refusal rather than physical singularity. A library midpoint
preconditioner together with a centered parameter residual enclosure passes the unchanged
local Newton checks at a neighborhood covering the original derivative step. This numerical
seed now precedes the retained scalar fallbacks; it supplies no permission before uniform
chart and all-regime proof checks. All 33 linked proof controls pass. Full competitor
exclusion and the original scientific cases still require current verification.

The assembled campaign also identified two obsolete metadata-allocation assertions;
the corrected tests distinguish shared Arrow payloads from separately retained checked
metadata and pass. Native root-response and physical NLP tests now require exact original
stationarity and complementarity for Ipopt/POUNCE, rather than the lower qualification
previously caused by unsupported presolve multiplier recovery. SCIP's distinct expectation
and all response, active-bound and scientific assertions remain. The PR Hessian diagnostic
now subtracts each original Jacobian row before weighted accumulation, preventing large
unchanged pressure contributions from obscuring small changes. Its rows, weights, derivative
steps and agreement thresholds are unchanged, with a controlled cancellation regression.

Execution pause, 2026-10-04, requested by the maintainer: all active agent assignments are
integrated directly in the existing `main` working tree, with no merge conflicts. The bounded
predictor review found no material correctness issue. The broader assembled/Python and
original PR campaigns were deliberately stopped; their completed slices and logs are partial
evidence, not current full qualification. A publication protection test timed out in the
concurrent default run and passes in isolation and the native run; its cause is not established.
The remaining order after resumption is original-budget PR scientific verification, any
resulting functional repair, final stable assembled/seed/domain qualification, required
dev-profile measurements, bounded closure and obsolete-worktree assessment/removal. None
of those pending campaigns is running in the background.

### Functional reconciliation, 2026-10-05

Current examined tree: `ad665a0222551196b1160e426f5242361215a6a0` plus the integrated working-tree
repairs. The functional obligations F01–F17/IP01–IP12 now have present corrections and named
passed controls. The subsequent automatic-composition obligations and their current receipts are
owned by [25n](25n-automatic-simulation-solve-pipeline.md#outcome). Both plans remain in progress;
25k owns K3/K4/K5 and final disposition. This functional reconciliation does not grant full
scientific acceptance, required measurement closure or an initially clean full gate.

**Tested:** the coordinator's final `just native-test --profile ci` with its archived configuration,
pinned nightly/dev build, linked native graph and explicit force-validation passed all 2,823
selected/results against baseline zero in
`build/plan25k-20261005/native-qualified-complete/checks.json` and `native-test.xml`.
Nextest run `81ef7be6-8197-47c8-9705-3fc7d01feb6f` has zero failures, errors, selected skips or unrun
controls; the native run took 834.667 seconds and assessment took 859.069 seconds. Two ignored,
unselected standalone KKT/Clarabel timing controls remain outside this gate. The reviewer verified
that all 1,839 relevant Rust-product inputs and the environment were unchanged; the global
source-context difference consists of documentation changes and does not invalidate this product
receipt. This is the primary current enclosing native evidence for the named functional controls.

The repair history remains composite: the initial full run recorded 13 failures, one timeout and
34 unrun controls; the next full receipt in `native-qualified-final/` recorded 2,821 passes and two
stale codec failures; `codec-repair.xml` recorded their 2/2 focused pass before the clean full rerun.
Those failed/partial receipts retain their original results. The full authored manifest, launched
at 12:14 UTC, is still active with no terminal reports at this reconciliation. K3's remaining
scientific scope, required K4 measurements and K5 final acceptance remain pending with 25k.

The integrated closeout repairs preserve Constant-evaluation publication without inventing native
status/KKT fields; release completed block construction allowances while admitting actual retained
report capacity and escaping component lifetimes; require the enclosing recovery grant for fresh
multistart seeds while preserving the frozen original contract and explicit native start profile;
and derive assessment charging owners from the actual dispatched execution occurrence. Targeted
retention and multistart controls, followed by their enclosing native execution, establish those
functional corrections. Unknown reporting extensions retain the complete bound, not a zero-byte
claim. Resource reservations remain distinct from measured process memory.

The explicit direct fixtures for nested hints, second derivatives and regime crossings retain the
production provider path. Automatic composition may legitimately reconstruct and assess a complete
original case as Constant; it need not manufacture an outer native report. These fixtures bind one
actual-profile direct rung where the test specifically requires outer callbacks. Their authored
laws/bounds, `0.75`/`0.03125` derivative expectations and 20-iteration crossing observations remain.
Scientific campaigns retain original physical inputs, branch conditions, bounds and independent
acceptance budgets. Earlier inconclusive PR campaigns keep their original interpretation.

## Outcome (recorded after implementation)

### What was built

**Implemented, scoped Tested:** P0–P11 functional work and migration are present, and the
finding tables reconcile their corrections against named current controls. The subsequent actual
M8/M17/M18 automatic consumers and integration corrections are implemented with scoped execution
evidence owned by [25n](25n-automatic-simulation-solve-pipeline.md#outcome).
Plan 25k's K3/K4/K5 assembled scientific qualification, required dev-profile measurements and final
acceptance remain pending; these functional results do not close either plan.

**Tested (2026-10-04, targeted scope; zero-failure target):** `just unit-native-package
pse-backend-native native-solvers` with the selected useful-chart, projection, chart,
retention, finite-chain and actual-IBEX filters passed 33 controls. `just unit-native-package
pse-runtime pse-runtime/native-solvers` with the six explicit root-response controls,
`weighted_jacobian_difference_preserves_changes_beside_large_unchanged_rows` and
`original_pr_case_jacobian_matches_stable_value_differences` passed all eight tests.
`just native-test` selecting
`authored_physical_nlp_preserves_native_routes_and_original_qualification` passed one
test; selecting the repaired native codec and checked-reuse/single-input-concat tests
passed both tests after correcting a test-owned retained slice lifetime. These 44 selected
tests used the pinned dev build, Nextest CI profile, explicit `pse-relations/force-validate`,
native solver features, eight or four test threads and one BLAS/OpenMP thread. The original
PR derivative diagnostic is a passing derivative check, not qualification of the stopped
complete PR scientific case. Independent bounded source review found no material issue
with the chart predictor; it does not establish full scientific covering or K5 closure.


### A mistake made and corrected

Completed block results initially kept the large construction allowance for every retained
component. The completed-report capacity calculation now admits vectors, maps, strings, portable
warm seeds and typed failures while retaining a conservative complete bound for opaque extensions;
extracted components retain the escaping owner and failed construction releases its allowance.
Fresh multistart seeds initially bypassed the enclosing recovery grant and lost the screened native
start profile; original assessment charges also reused an owner across dispatched rungs. The grant,
frozen profile and execution-occurrence corrections now have actual positive/negative controls.
Constant publication now preserves evaluated values and feasible evidence without fabricating native
status or KKT fields.

### Deviations from the plan, deliberate

Provider-focused callback fixtures select a bound direct composition so they continue to prove
their intended production provider path under the new Auto default. This changes test setup, not
scientific laws, domains, bounds, derivative expectations or original acceptance. Automatic complete
reconstruction may qualify without a redundant outer native attempt. Required K4 measurements and
unexercised scientific campaigns remain limits owned by 25k, not implied functional defects.
Architectural decision changes retain their existing decision route.
