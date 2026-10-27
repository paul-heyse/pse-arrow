# Numerical workflows

**Group G7 of the [first comparative review](README.md), 2026-10-27.** Covers
[CBO-033](../codebase/numerical-workflows.md#cbo-033) to
[CBO-036](../codebase/numerical-workflows.md#cbo-036). This is a comparative judgment. It
authorizes nothing and does not replace a formal review that is due.

| Item | Value |
| --- | --- |
| Source inspected | `main` at `3e9a2525b`. Between the audit baseline `dbc4c9436` and `HEAD`, the only `pse-runtime` change is in `workflow/modeling/canonical{,/delta}.rs`, which no G7 conclusion depends on. |
| Consumed accounts | The codebase account, library accounts and `assessment-patterns.md` revision 2, all as published at `34f9f07ee`. The egobox-doe probe evidence as published at `2f06c3266`. |
| Standard | Core 3.4 with the efficient-architecture heuristics, process-simulator profile 1.5 (PS-10, PS-11, PS-12 and PS-09 apply here) and the pse-arrow binding. |
| Parity sources read | `external/idaes-pse` (2.13.0): `core/util/parameter_sweep.py`, `core/surrogate/pysmo/sampling.py`, `core/util/convergence/convergence_base.py` and `apps/uncertainty_propagation/uncertainties.py`. Pyomo 6.10.1 `contrib/parmest/parmest.py` in `.venv-parity`. |
| Evidence | Every target here is *Proposed*. Mechanisms are *Interface-checked* where this page names the inspected source. One probe is *Tested* and is cited where it is used. No quantitative claim is made. |

## Group summary

**The joint decision.** All four opportunities belong to one area: point sets and statistics
around fits and studies. Two questions settle most of them.

1. **Does the work complete a capability that pse-arrow already claims, or add a new
   category?**
   - **CBO-033 completes an in-scope capability.** The scope table marks IDAES parameter
     sweeps as *Implemented*. The IDAES 2.13 sweep, however, includes sample generation:
     `ParameterSweepSpecification.set_sampling_method` with pysmo's Latin hypercube, uniform
     grid, Halton, Hammersley, CVT and custom samplers. pse-arrow has no generator. Closing the
     gap needs no new dependency, because egobox-doe is already compiled into production
     through egobox-ego.
   - **CBO-036 also completes an in-scope capability, but it is not a minor change.** parmest
     estimates σ² when measurement errors are not supplied. The accepted ADR-0118, however,
     deliberately fixes the declared-σ statistical model and names "a fit without declared
     standard deviations" as a negative control.
   - **CBO-034 is a new category.** The parity reference (IDAES `uncertainty_propagation`) is
     first-order, as pse-arrow is. §19.8 says propagation is first-order only.
   - **CBO-035 is a maintenance question with a mostly false premise.**
2. **What does reproducibility mean (U2)?** This decides how heavy the CBO-033 target is, and
   how heavy any future sampling route for CBO-034 would be. U2 remains unresolved, and every
   affected target below has a branch for each answer. The evidence for that decision, without
   making it:
   - **Points are already identified by their realized values.**
     - A multistart rung's key hashes its realized seed values
       (`math/solves/multistart.rs`, `prepare_multistart`).
     - A study point's identity is the binding content identity of its admitted assignments
       (§19.3, §5.3).
     - The surrogate's options key hashes the bits of each initial point
       (`math/surrogate/library.rs`).
   - **A seed sits in identity only for randomness internal to a library.** The surrogate
     hashes `seed` and `threads`. It needs them because egobox's internal infill and GP-start
     randomness cannot reasonably be recorded.
   - **Both parity references record realized samples.** IDAES `ParameterSweepSpecification.to_dict`
     stores `samples`. Convergence evaluation writes the sampled points to a JSON baseline once
     and replays them.
   - **Downstream results depend on the CPU class.** The Gauss–Newton covariance comes from faer
     SVD (`workflow/fitting/oracle.rs`), and propagation's PSD admission decides on faer
     `self_adjoint_evd` eigenvalues (`workflow/uncertainty/psd.rs`). Both kernels dispatch SIMD
     and FMA at runtime. Bit-portable regeneration of *inputs* would therefore be the only
     portable stage in a pipeline whose *results* vary by CPU class. It would buy shared input
     identity across hosts, not identical results.

**Dispositions.**

| CBO | Disposition |
| --- | --- |
| 033 | **Recommend.** One Rust design producer over egobox-doe's unit-cube designs, feeding the existing explicit-point inputs. It has U2 branches. |
| 034 | **Retain current.** A new category with no consumer. The reopening route is recorded below. |
| 035 | **Retain current.** The premise is largely rejected, and the recorded pin trigger stays. |
| 036 | **Retain current.** This is a deliberate statistical contract and the leverage content is negligible. It is recorded as a parity completion the maintainer may select, with a bounded design. |

**Interactions with other groups.**
- **G4 (CBO-001, prepaid accounting).**
  - The design producer must charge its output and the library's workspace before it
    allocates. egobox's maximin kinds allocate O(n²) pairwise distances, an LOP-X001 shape.
  - The residual LM route already carries a hand mirror of the levenberg-marquardt QR buffer
    sizes (`matrix_bytes` in `workflow/fitting/oracle/residual.rs`). That mirror belongs to the
    CBO-001 joint judgment, and it stays because CBO-035 is retained.
- **G8 (Python boundary and registry).**
  - Python exposure of the producer is an ordinary pse-py function.
  - The U2-regenerate branch adds a descriptor to the study definition. That moves the
    `StudyRequest` and stored-definition document versions, which G8 owns.
- **G5/G6 (numerical strategy).** Multistart rungs are composed through
  `PreparedSolve::with_strategy`. The producer supplies their seeds. It does not change rung
  semantics, branch policy or the replay premise `ReplayProducerPremise::Multistart`.
- **U2.** Any group whose replay or identity rests on regenerated values should take the same
  answer.

**Shared prerequisites.**
- The U2 answer shapes the CBO-033 target. It does not change whether CBO-033 is recommended.
- CBO-034, if it were ever reopened, depends on CBO-033's producer and on U2.
- CBO-036 is independent of U2, because its statistics are deterministic.

**Reinforcing and competing choices.**
- **Reinforcing.** Choosing egobox-doe for CBO-033 keeps the ndarray 0.16 / rand 0.8 universe in
  production. egobox-ego already holds that universe, so the choice reinforces CBO-035's
  retention and adds no new universe.
- **Competing.** A Python scipy.stats.qmc producer competes with the Rust producer. Adopting
  both would recreate the fragmentation that CBO-033 exists to prevent.

**Priority by consequence.** This order is separate from prerequisite order.
1. **CBO-033.** It completes a listed parity capability with no new dependency, and study
   authoring uses it immediately.
2. **CBO-036.** It is a real parity gap, but it is a product-scope decision.
3. **CBO-034.** It is a new category with no consumer.
4. **CBO-035.** It is a revisit trigger only.

---

<a id="cbo-033"></a>

## CBO-033 — No producer of sampling or experimental designs

### Responsibility and target intent

The operation produces a finite, bounded, reproducible set of points over named physical
coordinates. Studies, multistart rungs and surrogate initial designs then consume that set as
ordinary explicit inputs. The producer owns only construction. The author owns which
coordinates are varied, their ranges and the point count. The declarations own the admissible
bounds.

**Who actually consumes point sets today.** This corrects the opportunity's consumer list.
- **Studies.** `StudyInputs::Explicit(Vec<StudyPoint>)` in `workflow/study_variants.rs` and
  `StudyPoint.overlay` in `workflow/study.rs`. These are the only consumers with public Rust and
  Python authoring surfaces. Sweeps and convergence-style evaluations are studies.
  `StudyInputs::Lazy` covers structural Cartesian variants selected by ordinal, not value
  designs.
- **Multistart rungs.** `MathService::prepare_multistart(original, seed: BTreeMap<SemanticId, f64>, …)`
  freezes one complete seed per rung. `PreparedSolve::with_strategy` composes the rungs. At
  `HEAD` the only callers are in `multistart_tests.rs` (search: `rg prepare_multistart` across
  `crates/`). Python's `with_numerical_strategy` in `pse-py/src/workflow.rs` binds only
  declared-case rungs, so it cannot compose a multistart rung.
- **Surrogate initial designs.** `SurrogateOptions.initial: Vec<Vec<f64>>` (at least two
  points). `MathService::prepare_surrogate` likewise has only test callers.
- **Not consumers.** Profile chains are deterministic secant predictors seeded from their
  predecessor (§19.4). Global certification belongs to SCIP. Neither consumes a design.

**Promised versus incidental.**
- **Promised.** Points are admitted as typed physical assignments, converted once and keyed by
  their realized content (§19.3).
- **Promised.** Multistart refuses incomplete, non-finite or out-of-bounds seeds and never clamps
  them. Changing a start never changes the scientific target.
- **Absent.** No contract promises design generation. The parity reference for an
  *Implemented* scope row has it.

### Credible complete options

| Option | Description |
| --- | --- |
| A. Retain current | Every caller supplies its own points. |
| B. Rust producer over egobox-doe 0.35.1 | `SamplingMethod::normalized_sample` gives `[0,1]^d` designs from `Lhs::new(&xlimits).kind(LhsKind::Classic | Centered | Maximin | CenteredMaximin).with_rng(rand_xoshiro::Xoshiro256Plus::seed_from_u64(seed))` and `Random::new(&xlimits).with_rng(…)`. Local code maps unit values exactly onto physical ranges and discrete levels. Local code also builds a per-coordinate uniform grid, because egobox `FullFactorial` truncates and has no per-coordinate level control. ([egobox-doe/seeded-space-filling-designs](../libraries/egobox-doe/README.md)) |
| C. Python producer over scipy.stats.qmc 1.18.1 | Promote scipy from the `test` group to a runtime or extra dependency of `pse`, and use `qmc.LatinHypercube`, `qmc.Sobol`, `qmc.Halton`, `qmc.scale` and `QMCEngine.integers`. ([scipy-qmc/design-engines](../libraries/scipy-stats-qmc/README.md)) |
| D. Integer-defined QMC with sobol_burley 0.5.0 | A new zero-dependency crate. Compose `parts::{sobol_rev, owen_scramble_rev, hash}` and keep the `u32`, which converts exactly to `f64`. Its points are random-access by index. ([sobol-burley/seeded-owen-scrambled-sobol](../libraries/sobol-burley/README.md)) |
| E. Local LHS over a direct rand_chacha stream | Bespoke stratified permutations. |
| F. egobox family move to egobox-moe 0.36.2 | Its `MixintSampling` default `LevelMapping::EqualSlices` gives fair discrete levels. It drags egobox-gp 0.37, egobox-ego 0.42 and `basin` along with it. |

### Comparative argument

**Semantic fit and ownership.**
- **B.** The library owns stratification, permutation and maximin selection. The repository keeps
  bounds authority, discrete-domain semantics, endpoint exactness and the recording of points as
  inputs. That split puts every domain rule with its domain owner.
- **C.** It has the richest catalogue, with strength-2 LHS, scrambled Sobol and discrepancy
  metrics. Its output is reachable only from Python, however, while two of the three consumers
  are Rust APIs. It would also add scipy to the runtime package against the current
  import-linter boundary that keeps test oracles out of `pse`.
- **E.** It is a small amount of code. It discards a library that is already compiled and
  qualified, and the user prefers the best existing library over bespoke code.
- **F.** It solves a problem (bound levels under-weighted in installed `MixintSampling`) that a
  few lines of exact local level mapping over `normalized_sample` solve without moving four
  crates.

**Composition and change locality.**
- B adds one pure function in `pse-runtime`, the owner §19.3 already names for sampling ("Sampling
  … retain their owners"). It moves `egobox-doe` from `[dev-dependencies]` to `[dependencies]`
  and adds a direct `rand_xoshiro = "=0.6.0"`, which is already in the lock.
- Its output goes through existing admission unchanged: study overlays, multistart seed maps and
  surrogate initial designs. No execution path, identity rule or document changes under the
  U2-replay branch.

**Execution (AP-07).**
- Classic, Centered and Random are O(n·d) in time and memory.
- `LhsKind::Maximin` and `CenteredMaximin` call `pdist` over all n points, five times. `Optimized`
  runs ESE iterations over `pdist`. These kinds are O(n²) in memory and time per evaluation.
  Study cohorts admit up to 10,000 members, so maximin kinds need an admitted size bound and a
  prepaid workspace charge. The default kind should be Classic.
- Source: `egobox-doe-0.35.1/src/lhs.rs`, `_maximin_lhs` and `_maximin_ese`.

**Failure and recovery.**
- **Library panics.** egobox panics rather than returning errors on a malformed `xlimits`
  (`ncols != 2`). `_maximin_lhs` calls `pdist(&lhs).min().unwrap()`, which panics when n < 2.
  The producer must refuse those inputs before calling. Panics unwind under `panic = "unwind"`,
  but they must not be a control path.
- **Bound mapping.** The library maps by `u * (ub - lb) + lb`, which is inexact at the endpoints
  (the probe saw `-1.4999999999999998` for -1.5). Multistart refuses out-of-bound seeds and never
  clamps. The producer must therefore map unit values itself: endpoints exact, membership
  verified, and a typed construction refusal instead of a clamp.

**Developer and operator machinery.**
- B removes the case-by-case invention the opportunity warns about.
- It adds a small provenance record (kind, n, seed and the algorithm or crate versions). The
  only work beyond that is a size bound for the maximin kinds.

**Choice.** B is the complete option. D is a strong *addition* only if quasi-Monte Carlo is
wanted. It is integer-defined, so it is platform-independent by construction, and its
random access by index suits lazy generation. LHS, random and grid cover the most-used IDAES
samplers. Halton, Hammersley and CVT parity are residual items, not part of this target.

### Revealing cases

- **A sweep variable with authored bounds `[0, 1]` and a grid or centered design touching
  `u = 1`.**
  - With the library mapping, the realized endpoint can differ from 1 by rounding. A multistart
    would then refuse the whole rung.
  - The target constructs endpoints exactly and verifies membership. It never clamps, so it
    preserves both the "refuse, never clamp" rule and declared-bound authority.
- **The egobox family is bumped from 0.35 to 0.36 to obtain the fair mixed-integer mapping.**
  - Under rust-random's minor-version value policy, the same seed now gives different designs.
  - *Replay branch:* earlier study definitions replay exactly, because their points are recorded.
    A new design differs honestly and carries new provenance. No guarantee weakens.
  - *Regenerate branch:* the same move silently breaks regeneration unless holds and golden
    hashes exist. This is why the branch below adds them.
- **A 10,000-point maximin design.** The quadratic distance workspace makes the kind unfit at
  that scale. The target defaults to Classic and admits maximin kinds only below a prepaid size
  bound.
- **A discrete (integer) sweep coordinate.**
  - egobox-doe has no discrete handling, and installed `MixintSampling` gives bound levels half
    weight.
  - The target maps discrete levels locally, with equal slices of the unit sample and exact
    integer arithmetic. Admissible levels come from the declared domain (§6.8).

### Disposition: Recommend

**Bounded target, under both U2 branches.**

- **Producer.** A pure function in `pse-runtime`, the workflow sampling owner.
  - *Inputs:* ordered dimensions, each with a target identity, a physical unit, a finite range
    and optional admissible discrete levels; the kind (`LhsClassic` as default, `LhsCentered`,
    `LhsMaximin`, `LhsCenteredMaximin`, `Random`, or `Grid(levels per dimension)`); the count
    `n`; and a seed.
  - *Outputs:* ordered realized points and a provenance record (kind, n, seed, generator
    algorithm and crate versions).
  - *Execution:* a fresh seeded sampler per design, using egobox-doe 0.35.1
    `normalized_sample` with `Xoshiro256Plus::seed_from_u64(seed)`. Never use the
    entropy-default constructors, and never call `sample()` twice on one instance.
- **Domain rules kept local.**
  - The default range is the finite authored bounds.
  - An explicit range must lie inside the authored bounds. A non-finite or unbounded coordinate
    without a finite range is refused.
  - Unit values map to physical values by a mapping that provably stays in range and is exact at
    both endpoints (a `lerp`-style formula arranged so that u = 0 and u = 1 return the bounds
    exactly), with wide ranges handled where `ub − lb` overflows. A membership check after such a
    mapping would only refuse points the producer built itself, so it is an assertion, not a
    refusal path. The producer never clamps. *(Revised after the numerical and solver methods challenge.)*
  - Discrete coordinates use equal-slice level mapping, with u = 1.0 (which egobox Classic LHS can
    produce exactly) sent to the last level.
  - Inputs are refused before egobox can panic: n < 2 for maximin kinds, or a malformed range.
  - Maximin kinds are admitted only below a size bound. The output (n·d·8 bytes) and a
    conservative bound on the kind's workspace are charged before allocation (G4); `pdist` holds
    about 12n² bytes (about 1.2 GB at n = 10,000). Maximin also runs on rayon's global pool,
    outside compute admission, so the producer runs it inside an admitted pool or admits its
    threads explicitly.
- **Consumer integration.**
  - *Studies:* points become `StudyPoint.overlay` assignments in the declared unit, so the
    existing admission converts them once and assigns identity.
  - *Multistart:* the complete seed map is the specification values overlaid with the design
    coordinates. Then `prepare_multistart` runs per point, followed by `with_strategy`.
  - *Surrogate:* points go into `SurrogateOptions.initial`.
  - *Python:* one pse-py function returns the realized points for `StudyPointDefinition`
    authoring.
- **Intended behaviour change.** None for existing callers. This is a new capability.
- **Machinery.** The repository gains one module and its provenance record. Nothing existing is
  removed, and per-caller generators are prevented.
- **Transition.** Trivial. The dependency moves from dev to normal, and no document changes under
  the replay branch.
- **Residual work, not in this target.** Halton, Hammersley and CVT parity. Convergence
  evaluation's truncated-normal sampling. A quasi-Monte Carlo kind through sobol_burley if one
  is wanted.

**If U2 means replay of recorded realized values:**
- The provenance record is attribution only. Identity stays on realized values, as today.
- Every non-Optimized kind may be offered. `Optimized` is acceptable as well, because its
  libm-dependent selection affects only producing a new design.
- No holds and no document version moves are needed.

**If U2 means bit-portable regeneration from a seed:**
- **The descriptor becomes a reconstructible premise.** The study definition carries the kind,
  seed, algorithm version and ordered dimensions *beside* the realized points. Regeneration
  must reproduce the recorded bits, and a mismatch refuses. Identity still keys on the realized
  values, while the descriptor joins the definition's content.
  - This moves the `StudyRequest` (currently Version 8) and the stored definition version (G8).
- **Kinds are limited to portable constructions.**
  - LHS Classic, Centered, Maximin and CenteredMaximin; Random; Grid.
  - `Optimized` is excluded because its φp criterion calls `powf`.
  - Static reading and the [probe](../../design_review/evidence/library-leverage-egobox-doe-determinism-2026-10-27/README.md)
    support these kinds on one host.
- **Value-defining crates are held.**
  - Hold `egobox-doe` 0.35.1, `rand` 0.8.8, `rand_xoshiro` 0.6.0 and `ndarray-rand` 0.15.0 in
    `[workspace.metadata.pse.pins]`, with the reason.
  - Golden-hash tests guard each kind.
  - Any egobox or rand move becomes a design-algorithm version change.
- **Quasi-Monte Carlo.** sobol_burley's integer pipeline is the preferred route here, because its
  values are fully defined by integer arithmetic and the crate has no dependencies.

**If contributing upstream is acceptable (U1):** an egobox-doe change returning `Result` for a
malformed `xlimits` and for n < 2 would make two local guards redundant. It is not available
until released and would arrive only with an egobox family move. The local guards stay
meanwhile.

### Rule impacts

- **Replay branch.**
  - The dependency changes need neither an ADR nor a review.
  - The new Python function changes no Python-boundary contract.
  - Documenting the producer at §19.3 goes through the architecture-amendment route, with a
    revision row in `blueprint.md`.
- **Regenerate branch.**
  - The study definition gains reconstructible content, and its document versions move. That
    needs at least a short ADR. A design review applies if the maintainer treats the descriptor
    as a change to the hashing contract.
  - The holds need a pin reason only.

### Evidence

- **Interface-checked:**
  - `egobox-doe-0.35.1/src/lhs.rs`: `normalized_sample`, `_maximin_lhs` with `pdist(..).min().unwrap()`, and `_maximin_ese`.
  - `math/solves/multistart.rs`, `math/solves.rs` (`with_strategy` and `with_automatic_products`), `math/surrogate/library.rs` (`SurrogateOptions` and its key), `workflow/study_variants.rs`, `workflow/study.rs` and `pse-py/src/workflow.rs`.
  - IDAES `parameter_sweep.py`, `pysmo/sampling.py` and `convergence_base.py`.
- **Tested:** the egobox-doe determinism probe, under its stated one-host conditions.
- **Proposed:** the target.

---

<a id="cbo-034"></a>

## CBO-034 — Only first-order uncertainty propagation

### Responsibility and target intent

The operation propagates a qualified parameter covariance to outputs, with validity scoped
honestly (PS-12). Two routes exist today:
- through a later solve's parametric sensitivities (`SensitivityRequest.propagation`);
- to a fit's own predictions (`FitUncertainty.predictions`).

Both compute `Σ_y = J Σ_θ Jᵀ` (`workflow/uncertainty.rs`). Validity is the conjunction of the
inputs' validity. Faer PSD admission accepts singular input, refuses indefinite input, and
withholds a negative propagated variance without clamping it. §19.8 states plainly that
propagation is first-order only. The IDAES parity reference,
`apps/uncertainty_propagation/uncertainties.py` (`quantify_propagate_uncertainty` and
`propagate_uncertainty`), is also first-order: parmest covariance with k_aug gradients.

**This is a new analysis category, not a completion.**

### Credible complete options

| Option | Description |
| --- | --- |
| A. Retain current | First-order propagation with the conjunction of input validity. |
| B. Prediction profile likelihood | Applies to a fit's own predictions. Extend the existing profile chains (secant predictor, halving, χ²₁ threshold) by pinning a *prediction* rather than a parameter. This needs a prediction-equals-target row in the fit NLP, so it runs on the NativeNlp route only. |
| C. Sigma-point (unscented) propagation | 2p+1 fully qualified solves at `θ̂ ± columns of √((p+κ)Σ)`. The square root comes from faer 0.24.4 `self_adjoint_evd` with `ComputeEigenvectors::Yes`, `Par::Seq`, extending the call PSD admission already makes. The sigma-point weights are local. |
| D. Monte Carlo or randomized QMC | Many fully qualified solves. The uniform points come from sobol_burley or a named PRNG. The normal transform uses statrs `Normal::inverse_cdf` (std libm) or explicit `libm` calls. Correlation comes from a faer square root. |

### Comparative argument

**Fit to the two consumers.**
- B is the most faithful extension for a fit's own predictions. It reuses machinery that
  already exists, rests on the same statistical assumptions as the parameter profiles, makes no
  RNG choice, and treats each point as an ordinary qualified fit with its original checks
  (PS-10).
- B cannot serve propagation through a different later solve. Only C or D reach that consumer.

**What C and D would need.** Each brings new scientific policy, not just leverage:
- what a sampled distribution claims (PS-12: case selection does not establish a distribution);
- how a failed sample or sigma point is published, because dropping it biases the result and
  withholding everything hides it;
- a reproducibility contract (U2);
- a per-request cost of many full solves.

**The library contribution is small.** faer root, statrs or libm normal transforms and
sobol_burley points are leaves. The substance is local statistical policy.

**No consumer has asked.** The opportunity says so, and the parity reference does not exceed
the current capability.

### Revealing case

A strongly nonlinear model can carry a first-order band that is misleading while every validity
row holds, because rank, second-order sufficiency and bounds are local conditions. Does
retaining the current route weaken a guarantee?
- **No.** The result says it is first-order (§19.8, ADR-0118), and PS-12 requires stating the
  linearization assumption, not removing it.
- **Applied to option C,** one infeasible sigma point exposes the missing policy. That confirms
  C is a new product, not a completion.

### Disposition: Retain current

First-order propagation with conjunctive validity is the declared contract and matches the
parity reference. Under the no-major-new-functionality rule nothing here is a clear-win minor
addition.

**Reopen** when a consumer needs nonlinear prediction uncertainty:
- **Prefer B** for a fit's own predictions. It is deterministic and needs no U2 branch.
- **Use C before D** for propagation through later solves.
- **Reproducibility of C or D.**
  - *If U2 means replay,* record the realized parameter samples as explicit study points.
    Faer and statrs at their pins are then acceptable.
  - *If U2 means regeneration,* use explicit `libm` crate calls for normal transforms. At
    libm 0.2.16, `exp` and `log` are pure software on x86-64 with SSE2 and on aarch64; the
    architecture overrides replace only the correctly rounded `sqrt` and `fma`. Factor the
    covariance square root with local scalar code instead of faer, and use an integer-defined
    point source. Even then the solve results stay CPU-class-dependent.

**Rule impacts on reopening:**
- A new relation family for sampled or profiled prediction intervals needs a short ADR.
- ADR-0118's first-order clause need not be superseded, because a new method can sit beside it.

**Evidence.**
- *Interface-checked:* `workflow/uncertainty.rs` (`propagate_checked`), `uncertainty/psd.rs`
  (faer `self_adjoint_evd`), IDAES `uncertainties.py`, and libm 0.2.16
  `src/math/arch/mod.rs` and `src/math/exp.rs`.
- *Proposed:* the reopening route.

---

<a id="cbo-035"></a>

## CBO-035 — Separate dense-algebra universes for least-squares and surrogate routes

### Responsibility and target intent

The residual LM route is a library-owned solver choice (ADR-0209, proposed). It is selected
automatically only for unconstrained, unboxed fits with exact weighted residuals and
responses. Its criteria are part of fit identity (`ResidualLmCriteria` version 1:
`30·ε` relative cost, step and cosine; `FitProfileV5`). The surrogate route uses egobox, which
works on ndarray. The opportunity asks whether either carries an algebra family that is not
worth its cost.

### Premise check

- **nalgebra.** nalgebra 0.35.0 is required by `diffsol-la`, `num-dual`, `feos`, `feos-core`,
  `quantity` and `xtask` as well as by `levenberg-marquardt` and pse-runtime. nalgebra 0.33.3
  arrives only through statrs defaults, which egobox-moe enables.
- **ndarray.** The lock carries two lines: 0.16.1 through egobox and 0.17.2 through `numpy`
  0.29 and `rustworkx-core` 0.18.1.
- **pse-runtime's own use.** Its direct nalgebra use is `oracle/residual.rs` plus the
  test-only `oracle/lm_inquiry.rs`. Its ndarray use is the surrogate boundary. Both are copy
  points, never mixed arithmetic.
- **Conclusion.** Removing the LM route would remove no family from the build. The premise
  "it carries a dependency family for a small, gated route" is false. What remains is a git pin
  whose revisit trigger is already recorded.

### Credible complete options

| Option | Description |
| --- | --- |
| A. Retain current | Rust-cv `levenberg-marquardt` 0.15.0 at git `e67066cf13` on nalgebra 0.35. |
| B. Release swap | Replace the git pin when upstream issue #33 produces a release on nalgebra 0.35. This is the recorded pin reason. |
| C. basin 3.1.0 | LM, `BoundedLevenbergMarquardt` and `TrustRegionReflective`, on the `Vec` backend (scalar) or `faer_v0_24`. It could lift the unboxed-only restriction. |
| D. Delete LM | Use NativeNlp Gauss–Newton only. |
| E. Local faer LM | A custom implementation. |

### Comparative argument

- **C is not a clear win.**
  - basin documents that it does not reproduce MINPACK's algorithm or stopping behaviour, and
    it has no switch to turn scaling off. Adopting it is a deliberate change to fit identity
    (`ResidualLmCriteria` v2, a new `FitProfile` version) and to the results of existing
    automatically routed fits.
  - Its faer backend would bring runtime-dispatched FMA kernels into solves that are scalar
    today.
  - It adds `rand_chacha` 0.10, and it would coexist with the `basin` ^1.13 that an egobox
    family move brings.
  - Bounded fits already have a qualified class-correct route (NativeNlp through Ipopt or
    POUNCE), so lifting the box restriction regresses nothing that is missing today.
- **D removes a distinct solver choice** that ADR-0209 adopted for measured reasons. That is a
  regression.
- **E re-implements solver internals** that a qualified library owns (PS-09).
- **A keeps** the scalar, CPU-deterministic MINPACK-style QR and every recorded fit identity.

### Revealing case

A fit with a single finite parameter box routes to NativeNlp today. Under C, automatic
routing would move it to bounded LM. The result and the recorded method change for a user who
changed nothing. Policy would move from the fitting owner's eligibility rule to a library
adoption. That is the regression the no-compromise rule forbids, unless the change is a
separately decided method extension.

### Disposition: Retain current

- **Keep** the git pin until a release lands (B is the existing trigger, with no new decision).
- **Bounded residual methods.** If they are ever wanted, they are a separate functional decision
  under ADR-0209. basin 3.1.0 would then be the candidate, on its `Vec` backend.
- **If contributing upstream is acceptable (U1):**
  - Contributing `statrs = { default-features = false }` to egobox-moe would remove nalgebra
    0.33 from the graph. It remains unavailable until released, and an egobox family move must
    precede it. That move conflicts with CBO-033 if reproducibility means regenerating from a seed
    (U2): CBO-033 holds egobox-doe 0.35.1 so seeded designs regenerate bit-identically, so the
    family move would need a design-identity version change. Under replay of recorded values there
    is no conflict. *(Revised after the omission and preservation challenge.)*
  - rust-cv PR #28 (IEEE-754 zero tolerance checks), if merged, would change termination
    semantics. Absorbing it means a version change of the criteria.

**Rule impacts.** None while retained. Adopting C would amend the proposed ADR-0209 and move fit
profile identity.

**Evidence.** *Interface-checked:* `Cargo.lock` reverse dependencies, `Cargo.toml` pins and the
`[workspace.metadata.pse.pins]` reason, `workflow/fitting/residual_policy.rs`,
`oracle/residual.rs` (`request_eligible`, `matrix_bytes`), the `#[cfg(all(test, feature = "solver-ipopt"))]`
gate on `lm_inquiry`, and the [basin](../libraries/basin/README.md) and
[levenberg-marquardt](../libraries/levenberg-marquardt/README.md) accounts.

---

<a id="cbo-036"></a>

## CBO-036 — Fit statistics assume absolute sigma; no estimated-variance path

### Responsibility and target intent

The fit's covariance, Wald and profile intervals, and propagated prediction bands all state
their statistical model honestly. That model is currently declared absolute σ:
- §19.4: "The declared standard deviations are taken as absolute, so no residual variance
  rescales the covariance."
- Admission refuses an included observation without a finite, positive σ
  (`workflow/fitting/modeling.rs`).
- ADR-0218 adds that unknown uncertainty can never become an included weighted observation.
- The accepted ADR-0118, item 2, makes "weighted least squares with a declared standard
  deviation" the declared statistical model and lists a fit without declared deviations as a
  negative control.

**Parity.** parmest in Pyomo 6.10.1 supports the missing case. With the `SSE` objective and no
measurement errors, `cov_est` estimates σ² = SSE/(n − l) and scales the covariance.

**This is a completion within the in-scope parameter-estimation capability, not a new
category. It is not minor, however.**

### Credible complete options

| Option | Description |
| --- | --- |
| A. Retain current | The absolute-σ model only. |
| B. Declared estimated-scale model | Opt-in. The fit declares the model `relative_estimated`, in which σ are relative weights. An omitted σ means a weight of one in the response's own unit, declared explicitly and never treated as zero. Because a single pooled s² adds residuals in different units, B must either require σ as a relative weight in each response's unit, estimate one scale per response quantity kind, or restrict omitted σ to single-kind fits (PS-01). *(Revised after the numerical and solver methods challenge.)* The rest of B is set out in the next table. |

What option B changes, and what it reuses:

| Element | Change under B |
| --- | --- |
| Residual variance | s² = 2f*/(n − p), with n the included scalar observations and p the free parameters. When n ≤ p the result is withheld with a new reason. |
| Covariance | Scaled by s², with the scale and degrees of freedom recorded. |
| Wald intervals | statrs 0.18.0 `StudentsT::new(0.0, 1.0, ν).inverse_cdf((1+level)/2)` replaces the normal quantile. |
| Profile likelihood | The statistic becomes (S(θ)−S(θ̂))/s², compared against `FisherSnedecor::new(1.0, ν)` (equivalently t²). The chain machinery is unchanged. |
| Propagation | Inherits the scaled covariance and names the assumption. |
| Library arguments | Guarded before each call, because both statrs constructors panic on out-of-domain input. |

### Comparative argument

- **What the library does.** statrs contributes one quantile call per interval. Everything that
  makes B statistically sound is local: the variance model, degrees-of-freedom accounting,
  covariance scaling, the profile statistic, withheld semantics and typed refusal. The
  leverage content is negligible.
- **What B touches.**
  - One accepted ADR clause, which would have to be superseded.
  - Fit declaration semantics, through a registry change to the authored fit relations.
  - Fit profile identity.
  - Four consumers: covariance, Wald, profile and propagation.

  It is an opt-in and regresses nothing. Its machinery, however, is that of a mid-size product
  change.
- **Model inadequacy.** An estimated scale absorbs lack of fit into "noise". The honest
  statement is a validity row naming that assumption, which is again new policy.

### Revealing case: blind parity would encode a questionable definition

parmest 6.10.1 sets `n = self.number_exp`, which `_count_total_experiments` computes by counting
unique data points. *(Corrected after the numerical and solver methods challenge: the first pass
said it counts experiments.)* A 50-sample transient therefore gives 50·k, as locally. The real
divergence is where several outputs share one data point, or several scalar outputs belong to one
experiment: parmest counts them once, while the local owner counts observations.
- **Where parity hides it.** The repository's parity fixture has one output per data point, so it
  would agree.
- **Where it diverges.** An experiment with three measured outputs at each of 50 data points gives
  n − l of 150·k − l locally against 50·k − l in parmest.
- **Consequence.** Degrees-of-freedom accounting must be the local statistical owner's
  definition. Parity for this feature must use single-observation experiments or declare the
  difference. Matching the reference here would move policy to the wrong owner.

### Disposition: Retain current

The absolute-σ contract is a deliberate and accepted statistical model. The library leverage
premise is largely empty, because statrs supplies leaf quantiles only. Option B is a parity
completion, but it is not a clear-win minor addition, so this review does not recommend it on
leverage grounds. The maintainer may select it through the functional and decision routes. In
that case B above is the bounded design, and its degrees of freedom are defined per included
scalar observation. The choice does not depend on U1, U2 or U3.

**Rule impacts if selected:**
- Supersede ADR-0118's statistical-model clause with `just adr-supersede`. That needs an ADR,
  with design review at the maintainer's discretion.
- Amend §19.4 and §19.8 through the architecture route.
- Version the authored fit relation, `runtime.parameter_covariances` and the fit profile
  (new relation-family versions within the registry route).

**Evidence.**
- *Interface-checked:*
  - `workflow/fitting/covariance.rs` (`wald`, the module's statistical-model note);
  - `workflow/fitting/profile.rs` (`ChiSquared::new(1.0)` threshold, `√(2Δf)` statistic);
  - `workflow/fitting/modeling.rs` (the σ refusal);
  - the `lm_inquiry.rs` unknown-σ test;
  - ADR-0118 and ADR-0218;
  - parmest 6.10.1 `parmest.py` (the σ² estimation branch and `n = self.number_exp`);
  - `python/pse/parity/tests/test_covariance_parity.py`;
  - statrs 0.18.0 `src/function/beta.rs`, whose special functions call inherent `f64::ln` and
    `exp`, so they use std libm whatever num-traits features are selected.
- *Proposed:* B.

---

## Pattern notes

LOP-X006 was sufficient to frame the group. This review refined it in [additional patterns](../assessment-patterns.md) revision 3:
- **Split by role.** Separate recordable *design* randomness from *library-internal* randomness.
- **Downstream ceiling.** A bit-portable stage feeding a stage that varies by CPU class yields
  results that vary by CPU class.
- **Kernel-derived admission decisions.** An admission decision made from SIMD kernel output is
  itself an amplifier.

Two base-framework notes for LOP-12 are recorded in that output's maintenance notes:
- a parity reference's helper can be absent behind an *Implemented* scope label;
- a library may supply only leaf computations, in which case the opportunity is a policy
  decision.

## Limits

- **Not run.** basin and sobol_burley were not compiled. The cost of the maximin kinds was not
  measured, and cross-architecture regeneration was not tested.
- **Not inspected.**
  - Study admission's bounds checks on parameter assignments. The target places that
    responsibility on the producer regardless.
  - The Python `StudyPointDefinition` authoring code beyond its type stubs.
  - Convergence-evaluation parity in pse-arrow.
- **Searches.** "No production caller" claims rest on `rg` over `crates/` and `python/pse` at
  `HEAD`. Generated code and tests were excluded where stated.
