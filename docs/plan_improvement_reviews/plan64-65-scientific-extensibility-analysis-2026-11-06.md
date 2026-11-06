---
title: "Plan64/65 scientific extensibility analysis"
status: proposed
date: 2026-11-06
doc_role: review
doc_topics: [design, libraries, numerical-execution, current-work]
---

# Scientific extensibility across Plan64 and Plan65

This supporting review recommends seven coordinated amendments to the remaining
scientific footprint. Their common purpose is to make additional physical families and
analysis modes cheaper to deliver without weakening physical meaning or scientific
acceptance. They are proposals for the [principal combined review](plan64-65-integrated-target-and-library-leverage-2026-11-06.md), not adopted plan
changes, architectural decisions, implementation, or numerical qualification.

## Basis and coverage

The inspected science checkout is `dfdc11c65fbbd0a317f260c23f3ef89716f44656`;
the root checkout observed at review entry was `6045168110f5f5948df43c4f642ea1737751675b`.
Science includes source commit `a860539c9` for original fixture observation. Existing
uncommitted FeedFlash source and controls were inspected as incomplete work and preserved.
Its matching generated adapters and repaired numerical controls were not qualified by
this review. No builds, tests, provider preparation, or numerical experiments were run.
The conclusions below are source-grounded proposals; benefit and performance are unmeasured.

Coverage spans [64D](../plans/64d-scientific-assets-and-equipment.md),
[64E](../plans/64e-inventory-distributed-and-cyclic-physics.md),
[64F](../plans/64f-specialized-process-families.md),
[64G](../plans/64g-analyses-studies-and-applications.md),
[65E](../plans/65e-scientific-portability-and-execution.md),
[65I](../plans/65i-scientific-operations-and-product-semantics.md), and
[65K](../plans/65k-scientific-artifacts-and-native-reconstruction.md).
The finite configured population remains owned by
[64H](../plans/64h-required-process-population.md); an equipment name, example, or
source enumeration does not qualify its configurations.

All six scientific-calculation overviews supplied capability leads:
[Symbolica](../libraries/scientific_calcs/symbolica.md),
[faer](../libraries/scientific_calcs/faer.md),
[Salsa](../libraries/scientific_calcs/salsa.md),
[ndarray](../libraries/scientific_calcs/ndarray.md),
[rand_xoshiro](../libraries/scientific_calcs/rand_xoshiro.md), and
[EGObox](../libraries/scientific_calcs/egobox.md).
Context7 documentation was resolved and queried for all six, then consequential details
were checked against maintained source or exact installed release source. Overview or
current documentation versions are not automatically the repository's versions.
In particular, ndarray's newer `ArrayRef` and newer rand_xoshiro state accessors must
not be assumed available at the pinned versions. The review uses core 3.4, efficiency
heuristics 1.0, ProcessSimulator 1.5, and test-assessment 1.2 as its qualitative standard.

Important existing strengths change the recommendation. The code already has sparse
symbolic/numeric factor separation, implicit response actions, field-granular Salsa
preparation, native DOE ownership, multi-output symbolic evaluation, SVD-based fitting
covariance, and horizon prediction with fallbacks. They are foundations to extend, not
missing capabilities to recreate. The November 6 selective Symbolica precision and
exact-block investigations in 65E also already cover much of that library opportunity.

The principal scientific contrasts are deliberately wider than one thermodynamic flash:

| Remaining population | Consequential variation the amendments must preserve |
|---|---|
| D02/D03 properties, chemistry and transport | State-coordinate reconstruction; ideal/caloric datum; actual phase/root; activity-potential derivatives; reaction energy accounting; electrolyte true/apparent composition and charge balance |
| D04/D05 equipment, HDA and methanol | Rating versus design; selected property/reaction closure; interconnected initialization and complete steady consumer outcomes |
| E02–E04 storage, distribution and cycles | Inventory versus flow; spatial boundary/source semantics; temporal accumulation; carried physical state; scheduled transitions and periodic closure |
| F02 steam/power | Pure-fluid phase and supercritical regions, wet-state limitations, enthalpy/entropy datum, pressure/heat equipment integration |
| F03/F04 gas-solid and separation | Distinct phases and material bases, heterogeneous transfer/reaction, speciation, rate-based contacting, membrane driving force and selective transport |
| F05/F06 TSA, SOC and gas networks | Cycle carryover, thermal/mass/charge coupling across regions, electrochemical closure, compressible inventory and transient linepack |
| G02–G06 inference, uncertainty, design and control | Shared/local parameters, identifiability, dependence/measure, discrete decisions, physical time/horizon coupling, cost/environmental assumptions and scoped outputs |

These are investigation and acceptance contrasts within the finite configured scope;
they do not enlarge its denominator. F03–F06 retain their specific chemical and physical
prerequisites even when E03 delivers a reusable spatial/temporal accumulation slice.

<a id="S01"></a>
## S01 — Compose physical domains through complete consuming models

**Proposed amendment:** organize remaining D/E/F delivery around a small set of complete,
contrasting consumers which expose missing shared semantics, and change the actual owner
of each exposed meaning once. Retain authored properties, reactions, equipment, and
child composition. Avoid either a family-name switch in Rust or a universal physics
ontology that must be populated before useful science can run. This follows AP-03,
AP-04, AP-05, and PS-01–PS-05.

The shared contract needs explicit material membership and phase allocation, storage
quantities, geometry, transfer/source terms, boundary conditions, datum conventions,
and operating programs where the selected family uses them. These are related meanings,
not synonyms: valid dimensions do not prove a constitutive law, a component inventory
does not define phase equilibrium, and a time schedule does not define event semantics.
Derivative eligibility and scientific validity must remain tied to the selected
formulation and its active physical region.

For E04, distinguish externally scheduled switches, state-triggered events, resets,
and periodic boundary conditions. A working smooth trajectory does not establish
event location, saltation response, or differentiability through a switching surface.
Prefer an explicit supported operating-program subset with refusal of unsupported
products to a generic dynamic flag that implies all these meanings.

The existing [selection closure](../../crates/pse-modeling/src/scientific_selection.rs)
already closes authored dependencies under a selected context. The existing
[temporal specialization](../../crates/pse-modeling/src/temporal.rs) admits child
hierarchy and owns the temporal coordinate. Extend these owners when an actual consumer
requires it; do not create another scientific catalogue or temporal graph.

The most informative next distributed consumer is constant-density aqueous component
inventory with explicit volume, flow, boundary composition and isothermal treatment.
The existing [one-dimensional control volume](../../packages/reference/process/models/control-volume-1d.pse)
supports spatial meshes and flux/source closure but deliberately refuses dynamic
operation. A native temporal accumulation extension should first establish local and
independently accumulated global balance. An Erlang response would be an exact
finite-volume/cell-cascade reference for its selected cells, not continuum-pipe accuracy
or qualification of a gas-solid, membrane, steam, or SOC family.

Compare three implementation choices at the first real seam: an authored child
composition using unchanged primitives; a bounded extension to the existing shared
inventory/spatial owner consumed by two contrasting families; or a new general physics
IR. Prefer the first two where sufficient. The third incurs broad compatibility and
scientific obligations that one transport example cannot justify. Only remove older
family adapters after their actual consumers and configured cases migrate.

<a id="S02"></a>
## S02 — Route mathematical representations, not equipment labels

**Proposed amendment:** make formulation-preserved mathematical structure determine the
native solution route and reusable response representation. This is a stronger
extensibility mechanism than adding one solver callback per new unit. It applies to
distributed E/F models, G fitting/control/design and 65E/K provider boundaries
(AP-02, AP-07, DP-13, PS-05–PS-10).

Preserve the distinction between a linear or conic formulation, a square nonlinear
system, a bounded/constrained smooth optimization, a differential/residual system, and
a mixed discrete problem. Their initialization, feasibility, derivatives, termination
and assurance contracts differ. Existing native solvers should consume their matching
class rather than every family being recast as an opaque nonlinear objective. An
authored equation's structural classification and the current provider's supported
capabilities must both agree before dispatch. A portable mathematical description
should retain enough structure for that choice without retaining a live solver handle.

Symbolica 3.0.1 already supplies evaluator instruction export, coefficient remapping
with precision, shared multi-output evaluation, and exact symbolic operations.
Relevant maintained owners include
[`export_instructions`](../../vendor/symbolica/src/evaluate/instruction.rs) and
[`map_coeff_with_prec`](../../vendor/symbolica/src/evaluate/evaluator.rs).
Use selective affine/rational elimination or specialization where it preserves original
coordinates and guards. Carry pivot/nonzero/domain assumptions into the consuming
formulation rather than treating algebraic equivalence as global physical equivalence.
Do not substitute plain symbolic Newton iteration for a bounded or globalized native
problem class. Expanding exact manipulation everywhere would introduce expression
growth and preparation cost without evidence that the affected consumers benefit.

For large implicit responses, the overlooked faer 0.24.4 surface is its library-owned
matrix-free machinery: [`LinOp`, `BiLinOp`](../../vendor/faer/src/operator/mod.rs),
[`lsmr` and `lsmr_scratch`](../../vendor/faer/src/operator/lsmr.rs), and the sibling
conjugate-gradient, BiCGSTAB and spectral algorithms. The traits accept multiple right
hand sides and explicit parallelism/scratch. Existing native response actions are
a plausible operator boundary; an operator experiment need not serialize a Jacobian
or invent a second factorization framework.

Compare retained sparse factors against matrix-free actions using the actual consumer:
many repeated right hand sides often favor factors; very large sparse state spaces may
favor actions and a suitable preconditioner. A general nonsymmetric operator does not
justify conjugate gradients, nor does iterative convergence establish a root,
conditioning certificate, or reliable covariance. Preserve the direct weighted-response
SVD in [fitting covariance](../../crates/pse-runtime/src/workflow/fitting/covariance.rs),
which deliberately avoids forming normal equations. Keep rank and covariance withholding
conditions explicit if a larger-response route is introduced.

The maintained faer arithmetic changes documented in
[vendor/faer-arithmetic.md](../../vendor/faer-arithmetic.md) constrain applicability:
the exercised binary64 arithmetic/dispatch closure is not blanket qualification of
every generic scalar or iterative method. Acceptance must compare original-coordinate
actions, residuals, conditioning behavior and workspace against a retained direct route
on discriminating problems, then measure the real large consumer. Delete bespoke
iteration or matrix copies only where this library route actually replaces them.

<a id="S03"></a>
## S03 — Reuse preparation by semantic invalidation boundary

**Proposed amendment:** extend existing incremental ownership to distinguish source
identity, physical specialization, sparsity/layout, numerical values, branch selection,
precision and native solver context. Reuse each only under its own validity condition.
This joins D/F source growth, G repeated analyses and 65K reconstruction without a
universal cache or a second durable mathematical authority (DP-01, DP-08–DP-12,
DP-18–DP-21).

[Granular modeling](../../crates/pse-compiler/src/workspace/modeling/granular.rs)
already uses field-level Salsa inputs, tracked phases, retention and heap accounting.
The opportunity is to expose expensive shared preparation at the right phase and
remove equivalent repeated local maps after consumers move. It is not to put every
iteration into Salsa or hold database references while long native work executes.
The pinned maintained Salsa 0.28.5
[source contract](../../vendor/salsa/src/lib.rs) restricts `specify` to a tracked
function with one tracked key, specified by the tracked invocation creating that key,
and disallows its combination with LRU. Those restrictions can support creator-owned
bulk derived facts; they cannot implement an arbitrary cross-owner memo table.

Native factor/compiled evaluator scratch has a different lifetime from immutable
preparation and from published scientific products. Keep 65K's passive canonical
description, live provider-free mathematical body, and prepared mutable native session
distinct. Salsa persistence, if deliberately selected later, would not make cached
results canonical storage or grant native execution permission. Long computations need
cooperative cancellation outside tracked queries; retained snapshots must not prevent
revision settlement indefinitely.

Compare query reuse, solver-local symbolic-factor reuse and direct bounded recomputation
for each actual changed input. A permeability change, mesh-topology change, phase-regime
change, precision change and parameter-value update should be deliberately contrasting
invalidation controls. Proposed benefit is less repeated work and clearer lifecycle;
neither is measured here.

<a id="S04"></a>
## S04 — Preserve physical coordinates across native batch and layout boundaries

**Proposed amendment:** use library-owned borrowed array/operator views and coherent batch
evaluation at mathematical boundaries, while retaining native physical coordinate
mapping outside numerical kernels (AP-04, AP-07, DP-13, DP-15, PS-01, PS-07).

At pinned ndarray 0.16.1, `ArrayView`, `ArrayViewMut`, `Zip`, `and_broadcast` and
`for_each` already support borrowed data and fused traversal; newer `ArrayRef` is not
a prerequisite. faer `MatRef`/`MatMut` and explicit scratch similarly support native
linear algebra. Prefer those mechanisms to per-entry allocation or round trips through
Arrow. Compare contiguous batches, strided views and explicit packing using a real
provider/mesh consumer, since zero copies do not automatically imply faster kernels.

The semantic adapter must preserve species/phase membership, unit/datum, ordering and
original coordinate identity. A ragged phase-composition population and a rectangular
mesh-by-sample array expose different contracts; they should not share a falsely
rectangular public domain representation. Provider preparation and coarse batches may
be reusable, but thread-local or cloned evaluator scratch remains mutable and owned.
Charge retained high-precision coefficients, factors, packing buffers and scratch to
their actual native lifetime, not only the visible output matrix.

Migration should remove proven duplicate numeric staging and allocation in selected
consumers, preserving boundary validation and refusing incompatible shapes. It should
not introduce a new tensor abstraction merely to conceal coordinate conversion.

<a id="S05"></a>
## S05 — Extend adaptive studies through existing engineering ownership

**Proposed amendment:** make adaptive experiment selection a native engineering-study
capability which can consume fitting, expensive simulation and design results. This
extends beyond importing a surrogate file or optimizing a single scalar callback
(AP-01–AP-03, DP-13–DP-17, PS-11–PS-13).

Pinned EGObox already exposes a promising separation. In ego 0.38.1,
[`EgorServiceApi::suggest`](https://docs.rs/crate/egobox-ego/0.38.1/source/src/solver/egor_service.rs)
accepts existing design and response arrays and proposes another design; its factory
supports mixed-integer space. In gp 0.36.4,
[`GaussianProcess::update`](https://docs.rs/crate/egobox-gp/0.36.4/source/src/algorithm.rs)
supports updating an existing model. These are concrete research candidates for an
outer native study loop, not evidence that all desired batch, asynchronous, constraint
or multi-output semantics are available unchanged.
The exact `update` source keeps theta and normalization fixed while performing a block
Cholesky update; it is not a full hyperparameter refit. An adaptive-study policy must
therefore compare incremental updates with deliberate refits when new observations
change the useful scale or model regime.

The existing [surrogate owner](../../crates/pse-runtime/src/math/surrogate/library.rs)
already uses `EgorSolver`, retained state, native work accounting, cancellation and
panic settlement. Replacing it wholesale with another workflow engine would discard
working ownership. Compare its existing stepped callback route, an ask/tell proposal
adapter under the same owner, and fixed DOE. Ask/tell becomes valuable when scientific
evaluations are independently owned jobs or human/laboratory observations arrive later;
fixed DOE remains simpler for predictable parallel campaigns.

Separate observation noise, posterior model variance, physical model discrepancy and
numerical error. None substitutes for the others. Library failsafe rejection,
imputation and viability strategies need explicit scientific disposition: an imputed
response must never silently become an observed physical result. Preserve failed
evaluations and their original experiment/attempt identity. Warm starts, checkpoints
and resumed study identity are separate from a fitted model's data provenance.
Discrete switches and mixed schedules also require an honest nonsmooth/mixed problem
class; a smooth surrogate search is not a global optimality certificate.

Joint inference needs a declared distinction between shared physical parameters,
experiment-local nuisance parameters, initial conditions, scheduled controls and measured
observations. Reusing one physical model across steady and transient experiments is
valuable only if those roles and their observation maps remain explicit. Independent
identifiability and residual checks must survive optimization; neither extra samples
nor a surrogate variance repairs structurally unidentifiable parameters.

This amendment should first consume one finite G study with an expensive native
evaluation, then a contrasting mixed/scheduled fitting experiment from 65I. Retire a
bespoke candidate-proposal loop only after matching proposal, failure, cancellation,
memory and observation semantics migrate. New persisted study fields require the
canonical storage/contract owners, not a parallel JSON result format.

<a id="S06"></a>
## S06 — Make campaign reproducibility independent of worker scheduling

**Proposed amendment:** bind stochastic stream and sampling design to semantic work units,
not transient worker order. Preserve physical measures, normalization and dependence
alongside seed/provenance when expanding G uncertainty and adaptive studies (DP-10,
DP-18, DP-22, PS-11).

Current native [sampling](../../crates/pse-runtime/src/math/sampling.rs) already admits
library DOE under original ownership and records pinned algorithm/coordinate-map
provenance. rand_xoshiro 0.6.0 `Xoshiro256Plus` supplies `jump` and `long_jump`; its
source conditionally supports serde, while the currently selected dependency does not
thereby acquire newer `state()` APIs. Exact API and feature choices must precede a
checkpoint implementation. See the
[0.6.0 source](https://docs.rs/crate/rand_xoshiro/0.6.0/source/src/xoshiro256plus.rs).

Compare persisting the generated DOE, cloning an admitted generator state, and deriving
work-unit streams through specified jump allocation. Persisting the generated design
often gives the simplest restart for a finite campaign; generator checkpointing is
useful when the remaining design is genuinely generated online. Either must also
preserve sampler-specific state, adaptive history, algorithm/version and transform.
An RNG snapshot alone cannot resume an adaptive or stateful sampler faithfully.
Separated streams are not proof of statistical independence, and a uniform sample in
normalized coordinates does not imply a uniform physical measure after a nonlinear
transform. Controls should vary worker scheduling and partial restart while requiring
the same intended work-unit design and original-result attribution.

<a id="S07"></a>
## S07 — Keep assurance products explicit across physics and analysis modes

**Proposed amendment:** use existing native product owners to distinguish observed values,
derivative eligibility, regularity/stability, numerical error, statistical uncertainty
and formal/exact claims. Carry the minimum demanded provenance and conditions; do not
add a universal readiness registry (DP-02, DP-22–DP-24, PS-02, PS-07–PS-13).

The current [completion owner](../../crates/pse-runtime/src/workflow/completion.rs)
provides task/step/point scope inside an enclosing run/attempt and retains coverage.
It deliberately withholds a governing permission for member-qualified points; this is
a bounded foundation, not implemented permission across every experiment/member.
65I I03 and 64g G01 still own that consumer integration. Structural Analyze evidence
can be valid with zero numerical members: qualify its inspected subject and operation,
without imposing a phantom physical-state prerequisite. Point-derived products still
need their exact applicable point permission. 65K separates passive observation from
prepared native authority. Preserve those distinctions for new responses.
Parent-owned proposed ADR-0251's invocation-bound provider
observations address actual winning regime and provider facts; do not infer stability
or selected phase regime from beta, source inspection, preparation, or logs. Original
fixture observation is useful passive evidence but is neither the mutable analysis
overlay nor proof of actual applied solver inputs.

Symbolica exact identity, a higher-precision repeated evaluation, a conditioning
estimate and a validated enclosure answer different questions. Error-propagating
floating point is not a rigorous error bound; basic ball arithmetic does not establish
transcendental enclosure; conversion of a series to an expression loses its remainder
contract unless retained separately. Select the least expensive mechanism that supports
the claim actually demanded, with explicit withholding where no supported certificate
exists. Preserve direct covariance qualification rather than labeling every returned
matrix statistical uncertainty.

Each complete family consumer needs independent physical expectations beyond library
self-consistency: component/energy balances and datum tests, limiting cases, validity
and stability boundaries, temporal/spatial refinement, and selected independent
reference comparisons. Preserve required unavailable data/optional prerequisites as
visible obligations. IDAES 2.13.0 remains isolated reference behavior, with no copied
implementation. Internal source enumeration and passing scalar controls do not close
a whole equipment or application population.

## Expansion triggers and integration handoff

The first amendment group fits the existing footprint: S01's bounded distributed
consumer, S03/S04's preparation and buffer ownership, and S07's demanded evidence.
S02's matrix-free route and S05's adaptive study should enter when a selected remaining
consumer exposes their concrete scale or ownership need. S06 belongs with that study's
campaign/restart contract. No dependency upgrade is recommended merely because an
overview describes a newer release.

Additional library investigation is justified by concrete triggers, not automatic scope
growth: Symbolica term streaming for demonstrated expression growth; structural graph
canonicalization for a real symmetry/redundancy problem, preserving physical identity;
tensor libraries for genuinely coupled tensor equations; residual DAE solvers for an
actual index/consistent-initialization gap; FeOS/CoolProp or another property backend
for a demanded envelope beyond the already selected CoolProp 8 HEOS Water closure
when native authored science is inadequate; and
HiGHS/Clarabel/SCIP when the preserved formulation really is linear, conic or mixed
integer. Each needs exact-release primary-source assessment and a consuming model.
No library or licence is rejected in this shortlist, and capability absence is not
inferred from an overview's omission.

Three contrasting workflows should drive detailed amendment selection: a large
distributed reactive/transfer model with conservation and stiffness; a mixed steady
and scheduled transient inference campaign with identifiable parameter roles; and a
multiperiod/control/design application with repeated solves and changing active sets.
These expose substantially different data, derivative, initialization, branch,
uncertainty, lifecycle and reuse requirements. Existing
[horizon orchestration](../../crates/pse-runtime/src/workflow/horizon.rs) already handles
sequential physical plant application, estimation/control, predictive KKT response and
fallbacks; the third workflow tests generality rather than inventing that capability.

Scientific authors should provide the actual physical population, constitutive limits,
coordinate map and independent expectations. Modeling/compiler owners should extend
selected specialization and preparation at proven seams. Native mathematical owners
should provide original-coordinate value/action/error semantics and explicit allocation
lifetimes. Contracts/codegen should project those existing meanings faithfully;
persistence/evidence owners should retain original inputs, observations and unavailable
versus empty distinctions. New kernel, relation or architectural contracts still need
their existing decision route before implementation. The principal review owns combined
recommendation and adoption handoff; this document does not revise any plan status.
