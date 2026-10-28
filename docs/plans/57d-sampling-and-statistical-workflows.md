---
title: "Plan 57d: Sampling and statistical workflows"
status: in-progress
date: 2026-10-27
adrs: []
review_sources: [docs/library_leverage/reviews/numerical-workflows.md, docs/library_leverage/reviews/native-solver-integration.md, docs/library_leverage/reviews/result-data-and-prepaid-accounting.md]
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, design, libraries, preparation, numerical-execution, validation]
---

# Plan 57d: Sampling and statistical workflows

## Purpose and consumed baseline

**Proposed:** supply one bounded Rust design producer for existing explicit-point consumers,
with exact physical/domain mapping and admission before library allocation. Preserve the
current first-order uncertainty and absolute-σ statistical contracts and retain the selected
residual LM solver. This companion develops CBO-033–036 and the sampling/residual-LM inputs to
shared CBO-001. The [coordinator](57-library-leverage-and-correctness.md) owns the complete CBO
dispositions; the package state below is this document's native execution owner. Plan creation
does not authorize a dependency, product, statistical-model or architecture change.

Consume [G7 numerical workflows](../library_leverage/reviews/numerical-workflows.md) and its
2026-10-27 reassessment at `93946f572`, including the corrected finite-bound mapping, unit and
scalar-observation distinctions. Product source is `4f52fd239`; the authoring checkout was clean
at `93946f572`. Input accounts and additional patterns revision 4 are at that revision;
`1540fd164` contains the new counterexamples. The
selected assessment standard is core 3.4 with the efficient-architecture heuristics and
process-simulator profile 1.5. Relevant authorities are blueprint §6.8, §19.3, §19.4 and §19.8,
ADR-0118, ADR-0209 and ADR-0218. Their current status and statistical meaning stay with those
owners, not with the comparative review.

The operator chose **U2 recorded-point replay**: replay preserves realized inputs. Seed and
generator metadata describe how a new design was produced, not a promise to regenerate its
bits on every platform or future dependency version. U1 includes conditional upstream options
without contact; U3 preserves SurrealDB as canonical storage. These choices remove the proposed
regeneration-descriptor/version migration from this target. They do not waive the standing
FMA-free arithmetic requirement developed in [57c C02](57c-native-solvers-and-arithmetic.md#c02).

## Focused foundation assessment

The missing producer is a useful bounded addition, rather than a reason to change study
execution. `StudyInputs::Explicit(Vec<StudyPoint>)` and `StudyPoint.overlay` already provide
public Rust/Python study authoring. Realized assignments are admitted once and keyed by content.
Multistart's `prepare_multistart` and surrogate `SurrogateOptions.initial` already consume
explicit inputs, but their current production-facing caller surface is narrower: at the review
baseline the preparation calls are test callers, and Python numerical strategy accepts only
declared-case rungs. A design producer must not pretend those APIs already provide a public
multistart workflow. It should feed their actual Rust preparation boundaries and exercise them
without quietly adding a new Python solve composition API.

The installed egobox-doe 0.35.1 already lies in the production closure through egobox-ego.
`pse-runtime` declares it directly only under dev-dependencies. Its
`SamplingMethod::normalized_sample`, seeded `Lhs::new_with_rng` and `Random::new_with_rng` give
the desired unit-cube construction without importing Python scipy as a runtime oracle. Local
code should own physical units, authored ranges, admissible discrete levels and replayed point
identity. Using library `sample()` would repeat its inexact `u*(ub-lb)+lb` mapping; using
`FullFactorial` would truncate an implicitly selected grid rather than honor per-coordinate
level counts. Those are semantic reasons to use normalized samples and a local explicit grid,
not reasons to rewrite LHS stratification or permutations.

The physical operation needs an admission owner. Classic/Centered/Random produce O(n·d)
output and workspace. Maximin variants compare five candidate designs through `pdist`, with
quadratic storage and unconditional rayon work; their small-input `min().unwrap()` also panics
for n < 2. A helper that merely catches a panic after allocating or charges output alone is
incomplete. Reuse the existing execution admission and pool identity, charge conservative live
workspace and output before construction and contain library rayon work in the admitted pool.
Do not impose smaller workstation CPU caps or treat a fixture size as a host limit.

Statistical policy remains separate from producing points. First-order propagation already
matches its declared scope and parity category. Removing residual LM would remove no algebra
family from the closure, while switching its algorithm would change fit identity and
termination. Estimated residual variance is a legitimate parity completion but changes an
accepted model, units, degrees of freedom and four statistical consumers. A leaf statrs quantile
call does not justify adopting that change as library cleanup.

## Target interface and semantics

One workflow-owned producer receives ordered coordinate specifications and a selected design
request. Conceptual inputs are target identity, declared physical unit, finite range and
optional ordered admissible discrete levels per coordinate; kind, seed and point count or
per-coordinate grid levels; and the existing admitted operation's limits/cancellation context.
Concrete type names are left to the runtime owner. Coordinate order and point order are part
of this producer's output contract; identities continue to derive from the resulting admitted
assignments, not ordinal position or a random seed.

The result contains complete realized points plus attribution: kind, seed, count, generator
algorithm/release and ordered coordinate descriptions needed to interpret them. Attribution
must travel with the returned design for callers that retain it; it need not become a new
mandatory StudyRequest or stored-definition field. Explicit realized values are sufficient for
existing recording/replay. No second canonical store, reconstructible descriptor, seed-only
identity or dependency value-stability hold is introduced.

Expose Classic LHS as the default, Centered LHS, Random and explicit per-coordinate Grid.
Include Maximin and CenteredMaximin only with the complete admitted workspace/thread route in
D01. `Optimized` is an optional inquiry, not automatically exposed just because U2 allows its
platform-dependent design selection: its repeated ESE/pdist work still needs a finite
source-derived bound and cancellation behavior. Halton, Hammersley, CVT, truncated-normal
convergence sampling and Sobol/QMC remain explicit residual capability choices, not claims that
this package completes all IDAES sampler parity.

The local mapping owns these guarantees:

- Default ranges use finite authored bounds. Explicit finite ranges lie within the coordinate's
  admitted domain. Unknown targets, wrong units, repeated target identities, invalid order,
  inverted/nonfinite ranges, inadmissible levels and extents overflow refuse before the sampler.
  For a fixed coordinate, permit a singleton admitted value without sending a degenerate range
  through library logic; it consumes one coordinate value per point and does not invent spread.
- A range-preserving nonfused mapping returns the lower/upper bound exactly for u=0/u=1 and
  handles opposite-sign/wide finite ranges without overflowing `upper-lower`. Choose and explain
  a stable lerp-style construction before coding; its invariants are established at this owner.
  No clamp is allowed. Post-mapping membership checks are invariant assertions, not a normal
  refusal route for a value the producer just constructed.
- Discrete coordinates map equal slices of the unit interval to the explicit ordered admissible
  level list, with u=1 selecting the last level. Integer level arithmetic and conversions must
  preserve authored discrete meaning; do not rely on a lossy f64 integer outside exact range.
  Use the existing typed assignment representation or refuse an unrepresentable target before
  effects. Categorical support follows the actual declared-domain boundary; do not encode a
  category as an arbitrary floating index.
- A grid has an explicit positive level count for every varying coordinate and the exact
  Cartesian product size from checked multiplication. The request either names that resulting
  count or verifies equality with it; do not silently truncate a product to n. Define stable
  coordinate-major/ordinal ordering and endpoint construction. A one-level coordinate uses its
  explicitly selected single level; do not invent an endpoint-versus-center convention.
- Duplicate realized points can legitimately result from a discrete design. Preserve point
  occurrences rather than silently deduplicating, and let existing assignment content identities
  express equal realized inputs. Producer provenance does not create a scientific distribution
  claim, remove downstream admission or imply every point has a feasible solve.

A future QMC producer would plug into the same unit-cube-to-domain boundary and output contract.
A new statistical method would consume those explicit inputs through its own permission and
scientific policy; it would not redefine what current first-order uncertainty claims.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| D01 | pending | [Bounded design producer](#d01) | Library producer, domain mapping, admission and actual consumer integrations pass targeted controls; retained/inquiry dispositions remain explicit |

<a id="d01"></a>

### D01 — One admitted design producer and explicit-point integration

Source: [CBO-033](../library_leverage/reviews/numerical-workflows.md#cbo-033).

**Prerequisite slices.** Consume the coordinator's U2 recorded-point decision, existing typed
coordinate/domain admission and 57a's shared CBO-001 charge meanings. A documented byte/work
formula and admitted pool use are needed before enabling quadratic maximin kinds. Existing
point admission, model identity and study recording owners are reused; no whole-store migration
is a prerequisite. Coordinate producer metadata and Python exposure with the declaration/boundary
companion rather than duplicating generation or codec policy.

**Library construction.** Move `egobox-doe.workspace = true` from runtime dev-dependencies to
normal dependencies, keeping the exact workspace `=0.35.1` pin. Add one exact workspace
`rand_xoshiro = "=0.6.0"` declaration and runtime use if needed; that release is already in the
lockfile. Reuse the existing rand-0.8-compatible `SeedableRng` import boundary. Check that only
intended lock inputs move. No egobox family upgrade, new ndarray universe, scipy runtime
dependency or rand family move is required.

Construct a fresh `Xoshiro256Plus::seed_from_u64(seed)` per design, pass it directly to
`Lhs::new_with_rng` or `Random::new_with_rng`, select an explicit kind and call
`normalized_sample` once. Avoid `new(...).with_rng(...)`, which first consults OS entropy;
avoid shared/cloned sampler state and repeated `sample()` calls that continue its mutable stream.
Pass only an internally formed valid `(d,2)` limits matrix. Use local physical/discrete mapping
and explicit grid generation. Keep library-owned stratification, shuffle and maximin selection.

**Resource and interruption behavior.** Derive checked conservative bounds for both retained
output and peak temporary arrays from installed `lhs.rs`, `random.rs`, `utils.rs` and their
ndarray operations, including candidate overlap, distance arrays, copied limits, ordering,
metadata and mapped output. The review's approximate 12n² distance figure is a recognition cue,
not a complete admission formula. Admission happens before each protected construction and
retained output keeps its charge while a consumer borrows it; release temporary reservations
when the library returns or fails. Do not reserve the same output again merely to convert a
consumer view, and do not return an uncharged alias. Integrate pure deterministic construction
with the existing operation owner rather than creating a second pool or lifetime service.

For maximin n ≥ 2, cap the request through its admitted work/byte bounds and run rayon work in
an explicitly admitted execution pool/thread context. Setting `RAYON_NUM_THREADS` globally is
not scoped admission. Inspect whether the exact library call offers internal polling; where it
does not, preserve cancellation before/after a finitely admitted call and state the bound on
the noninterruptible region. Do not claim prompt mid-call cancellation without evidence. A
failed/cancelled generation returns no partial design as complete input. For future larger/lazy
designs, reopen the interface rather than hiding a streamed generator behind today's complete
point-vector contract.

**Consumers.** Studies receive `StudyPoint.overlay` assignments in the declared unit, followed
by existing admission's one conversion and content identity. Python receives one pse-py producer
function returning realized values suitable for `StudyPointDefinition` authoring; update its
public typing/documentation and error conversion through the existing generator/declaration
owners where applicable. This function must work without canonical-store or solver startup.
Recording uses existing `Runtime.record_study` and recorded study start, keeping SurrealDB
canonical semantics unchanged.

For Rust multistart, overlay design coordinates onto the complete original specification seed
map, then call `prepare_multistart` for each rung and compose through `PreparedSolve::with_strategy`.
Preserve complete/nonfinite/out-of-bounds refusal, no clamping, unchanged scientific target,
branch permissions and `ReplayProducerPremise::Multistart`. For surrogate preparation, pass
ordered physical points into `SurrogateOptions.initial`, keeping its at-least-two-point and
options-key semantics. These are bounded integrations of existing Rust consumers; new public
Python multistart/surrogate workflow APIs require separately selected scope. Profile chains and
SCIP global certification do not consume designs and receive no migration.

**Deletion and final state.** No existing production generator is replaced at this baseline.
Do not create separate per-consumer generator implementations. Delete the dev-only duplicate
dependency declaration when moving it to normal dependencies; retain fixtures that exercise
the unchanged explicit-input APIs. Any ad hoc test generator moved onto the shared producer must
retain independent expected values for assertions rather than testing the producer against itself.
Existing callers supplying their own explicit points behave unchanged.

**Targeted acceptance.** Use analytic grids, exact integer/discrete facts and independent
endpoint/domain invariants, not a duplicate egobox implementation. Add controls for fresh-seed
repeatability in the same complete configuration, stable ordering, default Classic selection,
LHS strata where applicable, random dimensions, exact lower/upper endpoints, extremely wide
finite ranges, signed zero, singleton coordinates, invalid units/targets/ranges/levels, grid
product overflow, maximin n < 2 preflight, byte/work refusal before library allocation and
cancellation/temporary-charge release. Include a case with repeated discrete values and
preserved occurrences. Test that the entire returned design, not merely one unit sample, feeds
existing study admission and exact point-value recording/replay.

Use `just check-package pse-runtime` and targeted runtime units; check `pse-py` and its selected
Python producer units after exposure, retaining explicit Arrow force-validation on correctness
closures and exactly one Python test tier. Test pure design/admission without unrelated store or
native startup. Separately exercise existing Rust multistart and surrogate preparation consumers
with selected fixtures. Scope-end 57j selects the real Rust/Python study creation, recording,
replay and execution journeys; the sampler's unit tests do not qualify solving or publication.
Cross-platform seed regeneration and numerical speed are not acceptance claims under U2.

**Rule/architecture route.** A dependency declaration or ordinary pse-py function needs no ADR
by itself. Documenting the new producer at §19.3 goes through the architecture amendment and
revision-row route after operator confirmation of any rule impact. Do not add a mandatory
reconstructible descriptor or bump the current StudyRequest/stored-definition versions solely for this
recorded-point route. If implementation reveals new relation/kernel/boundary contracts, route
them through the coordinator before dependent work.

## Retained contracts and bounded optional inquiries

### CBO-034 — First-order uncertainty stays explicit

Retain `Σ_y = JΣ_θJᵀ` for later-solve propagation and fit prediction bands, conjunction of input
validity, singular-PSD admission, indefinite refusal and withholding of negative propagated
variance without clamping. No empirical distribution or nonlinear coverage is inferred from a
point set. The FMA-free EVD premise remains a separate 57c C02 obligation.

**Trigger:** an actual consumer asks for nonlinear prediction uncertainty. For a fit's own
predictions first compare prediction profile likelihood, reusing profile chains with an explicit
prediction-equals-target row on the NativeNlp route. For propagation through a later solve compare
sigma points before Monte Carlo/QMC: a faer covariance square root and 2p+1 qualified solves need
new scientific policy. Settling evidence must define the distribution/coverage claim, out-of-domain
and failed-point publication, honest partiality, solve cost/admission and validity. Dropping failed
samples biases the answer. Under U2 record realized parameter samples as ordinary study points;
portable normal-transform/seed regeneration is not required. Arithmetic policy remains binding.
A new result/relation family requires its decision route. No CBO-034 implementation is scheduled
by D01; the inquiry becomes executable only if the trigger/scope is selected.

### CBO-035 — Retain the measured residual-LM method and current algebra boundaries

Keep `levenberg-marquardt` 0.15.0 at git `e67066cf13` on nalgebra 0.35, its scalar MINPACK-style
QR, unboxed/unconstrained exact-residual eligibility, `ResidualLmCriteria` version 1 and fit
profile identity. Nalgebra and ndarray families serve other consumers, so deleting this route
would not remove an algebra universe. The residual LM `matrix_bytes` QR mirror remains a local
source-derived pre-allocation bound; contribute its owner/derivation to CBO-001, rather than
claiming that a retained-size measurement discharges it.

**Existing trigger:** replace the git pin when rust-cv issue #33 produces a compatible release.
Inspect the actual release termination behavior, especially any PR #28 IEEE-zero tolerance
change; if stopping semantics change, version the criteria/profile and rerun affected fit
controls. No automatic seed portability hold is needed for an egobox family move under U2,
but ordinary exact pins, scientific qualification and changed-result identity remain.

**Optional inquiry:** if bounded residual methods are explicitly wanted, compare basin 3.1.0's
`Vec` backend with the existing NativeNlp bounded route. Its algorithm/scaling/stopping behavior
is not a drop-in MINPACK replacement; it must be selected as a method change, with explicit
eligibility and identity decisions. Do not substitute faer-backed basin while C02 arithmetic is
unresolved. Conditional U1 work can remove egobox-moe's unnecessary statrs default-feature
nalgebra-0.33 dependency after a released family move; it is not a mandatory upgrade or authority
to contact upstream.

### CBO-036 — Retain absolute σ; estimated scale needs a distinct decision

Keep included-observation finite positive declared σ, absolute covariance, current normal/Wald
and χ² profile interpretation and downstream first-order propagation. Preserve ADR-0218's rule
that unknown uncertainty never becomes an included weighted observation. This is a deliberate
accepted statistical model, even though estimated scale is a parity completion opportunity.

**Trigger:** the maintainer selects an opt-in estimated-scale statistical model through the
functional/decision route. A bounded design inquiry then settles dimensional semantics before
implementation: require relative weights per response unit, estimate a scale per response kind,
or restrict omitted σ to single-kind fits. Pooling unlike dimensional residuals into one SSE is
not valid. Omitted uncertainty must never silently mean zero or an included observation under
the old model. Its explicit new meaning, if selected, is part of the declaration.

For a selected compatible model, define n as included scalar observations and p as free
parameters; withhold when n ≤ p. Record `s² = 2f*/(n−p)`, degrees of freedom and the model
assumption. Covariance, Wald intervals, profile likelihood and propagation all consume the same
model. Candidate leaf mechanisms are statrs 0.18.0 `StudentsT::new(...).inverse_cdf(...)` and
`FisherSnedecor::new(1,ν)` thresholds; validate probability domains before calls that may panic
and map constructor errors. These calls do not own the statistical policy or give portable
libm bits. Compare multi-output observations explicitly: parmest counts unique data points,
whereas this proposed local n counts scalar observations. A single-output parity fixture cannot
settle that distinction.

Selecting the method requires superseding ADR-0118's statistical-model clause, architecture
changes at §19.4/§19.8, authored fit relation and runtime covariance versioning, fit-profile
identity and consumer migration. Those are conditional proposals, not accepted changes in this
series. Quantile API familiarity and negligible library integration do not erase that decision
burden. D01 completion does not depend on choosing estimated scale.

Under confirmed U1, a fallible egobox-doe constructor/sample route for malformed `xlimits`
and maximin `n < 2` remains a conditional supplier improvement. No contact is authorized.
Adopt only after a compatible inspected release provides the typed refusal and actual producer
controls pass; keep local pre-call guards until then. A family move still follows its real
consumer/algorithm dependencies, even though recorded-point replay requires no seed-portability
version hold by itself.

## Evidence and completion limits

**Interface-checked:** decisive runtime study/multistart/surrogate declarations, the runtime
dev-dependency placement and installed egobox-doe 0.35.1 seeded constructors,
`normalized_sample` and maximin source were inspected. Context7 was consulted first for egobox;
its general examples include entropy-default constructors, so exact installed source governs
the deliberate direct-seeded choice. The source review and library account retain the exact
pin/release and statistical evidence for the optional inquiries.

**Tested, inherited only:** the retained egobox determinism probe exercised library `sample()`
on one host under its recorded build/thread conditions. It did not exercise this normalized
producer plus local physical/discrete mapping/grid, nor prove complete cross-platform
regeneration. **Proposed:** D01 and every conditional extension. **Not run:** product compilation,
producer probes/tests, native/provider workloads or broad qualification during this documentation
task. 57j owns selected assembled evidence after functional implementation. No sampler parity,
statistical coverage, whole-product qualification or measured benefit is claimed by authoring.

## Authorized D–F execution — 2026-10-28

The maintainer authorized implementation of D01, E01–E04 and F01–F06 through the
conversational execution approach. Work starts from clean `6d13fc986`. The existing
package owners retain status and acceptance; 57j owns selected assembled qualification.
Full powerset/no-default validation remains deferred to series end. No upstream contact,
data reset or formal ADR status change is authorized.
