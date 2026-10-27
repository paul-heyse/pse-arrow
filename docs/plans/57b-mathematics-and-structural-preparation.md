---
title: "Plan 57b: Mathematics and structural preparation"
status: in-progress
date: 2026-10-27
adrs: []
review_sources: [docs/library_leverage/reviews/mathematics-preparation.md, docs/library_leverage/reviews/result-data-and-prepaid-accounting.md]
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, design, libraries, preparation, numerical-execution, validation]
---

# Plan 57b: Mathematics and structural preparation

## Purpose, baseline and authority

**Proposed:** repair the cubic accelerator's incomplete root enumeration and hidden retention,
extend the checked structural boundary to implicit reconstruction, and make the remaining
mathematics supplier couplings explicit. This companion develops CBO-021–025 and the mathematics
contribution to CBO-001. The [coordinator](57-library-leverage-and-correctness.md) owns their full
scheduled dispositions and shared decisions; the package table below owns only this document's
execution progress. Creating this plan authorizes no production, dependency or architecture edit.

The consumed review is [G5](../library_leverage/reviews/mathematics-preparation.md), including its
2026-10-27 reassessment and conversion corrections, published at `93946f572`. The common source
baseline is `4f52fd239`; the clean authoring checkout is `93946f572`. Input accounts and additional
patterns revision 4 are published at `93946f572`; `1540fd164` contains the new counterexamples.
Source-review evidence is inherited with its
original scope, not rerun or promoted here.

The focused foundation assessment uses core 3.4, the efficient-architecture heuristics and
process-simulator 1.5 selected by `standard.toml`. Contracts remain at blueprint §7, §14.3.2,
§14.4 and §15.3, ADR-0082 and ADR-0212. Relevant source owners are
`pse-math/src/implicit_cubic.rs`, `implicit_reconstruction.rs`, `library.rs`,
`library/admission.rs`, `native_compiled.rs`, `convexity.rs`,
`pse-structural/src/incidence.rs`, and runtime `math/jobs.rs` and `math/solves/derived.rs`.
These are repository source paths under `crates/`, not new module prescriptions.

## Focused foundation assessment

The cubic supplier chain has a concrete correctness defect. Symbolica/Numerica 3.0.1's
`isolate_real_root_intervals` can emit nested intervals across different square-free factors;
`refine_root_interval` then receives an interval containing two roots. The retained containment
probe demonstrates a debug panic and release duplicate/missing roots for
`12x³ − 68x² + 123x − 72 = (3x − 8)(2x − 3)²`. The fallback through
`isolate_real_roots`/`IsolatedRoot::refined` populates a process-global cache without eviction.
A changing-parameter sweep is therefore a relevant failure case even though each cubic is small.
The repair should keep root counting with Symbolica and retain local original-residual and
branch acceptance. Replacing the whole accelerator or accepting only the first root would lose
useful behavior.

The numerical output promise is narrower than the initial review remedy. The consumer needs a
finite representative of each distinct real root and exactly one original-residual-verified
candidate in the declared branch. It does not promise a correctly rounded binary64 root.
Default to that contract. Correct conversion of a chosen rational representative is a separate
problem from correct rounding of the algebraic root. Installed Numerica 3.0.1
`domains/rational.rs::to_f64` rounds numerator and denominator independently before division for
its fixed-width variants. Its 53-bit multiprecision fallback also does not establish correct
subnormal binary64 conversion. Neither equal `to_f64` endpoints nor symmetric half-ulp cells
are a sound independent certificate.

The structural gap is smaller. `incidence::analyze_inner` already checks matching/DM/BTF
results; reconstruction independently calls the same pounce sequence but trusts block order
while propagating retained-parameter support. Wrong order can silently omit derivative support.
Existing production math jobs already provide the qualified stack and case-size premises.
Extracting a checked dense owner is preferable to a second stack-owning service: a nested
32 MiB thread would require a second reservation and supplies no new semantics. A
`pse-math → pse-structural` dependency is acyclic at this baseline. Domain graph construction,
SemanticId ordering and source-restricted propagation remain distinct responsibilities.

Predict/admit/build/reconcile is already the right mathematics admission shape. Symbolica's
operation counts and exported instructions are public inputs to local private-algorithm mirrors;
actual products are checked afterward. Reconciliation catches drift but cannot undo an allocation
inside an underestimated library call. Retained-size derives do not solve that problem.
The shared CBO-001 accounting work in [57a](57a-result-data-and-prepaid-accounting.md) owns the
size taxonomy and common B-tree helper; this companion supplies math consumers and derivations.

## Target and preserved contracts

1. Cubic isolation operates on `u = p / gcd(p,p′)`, enumerating distinct roots once. Symbolica
   supplies exact square-free reduction and interval isolation. One local, budgeted exact bracket
   refiner supplies binary64 candidates. Original guards, residual tolerance, branch membership
   and uniqueness stay in `CubicWorker::solve`; repeated roots remain values but cannot become
   regular IFT jets. No approximate sign evaluation or silent finite-difference path is added.
2. One non-spawning dense-decomposition function in `pse-structural` owns the pounce sequence,
   limits, stage cancellation and result invariants. Callers retain stack admission and their
   semantic interpretation. Reconstruction gains typed refusal on malformed or unsupported
   decomposition; it does not receive all-pairs transitive closure.
3. One Symbolica hold names live private-behavior couplings and their drift controls. The
   existing abort binding and reconciliation remain until a released upstream capability meets
   the complete consumed contract. U1 permits conditional upstream options in the plan; it does
   not authorize contacting upstream or treating unreleased APIs as installed capabilities.

A new implicit branch reuses the checked decomposition and retains its own support propagation.
A future Symbolica release reopens the hold's exact source models. Raising the case limit above
100,000 rows reopens the matcher implementation; it does not automatically remove stack charges
from KINSOL or native-child consumers that also use `MATCHING_STACK`.

## Packages and dependency order

| Package | Status | Owner | Completion |
|---|---|---|---|
| B01 | implemented; joined qualification pending | [Cubic repair](#b01) | Complete bounded enumeration, candidate conversion, admission and old-path deletion; targeted controls pass |
| B02 | implemented; joined qualification pending | [Checked decomposition](#b02) | Both consumers use the checked non-spawning owner; targeted controls pass |
| B03 | implemented; joined qualification pending | [Supplier coupling and bounds](#b03) | Hold and math bound consumers integrated; residual/upstream conditions retain explicit dispositions |

B01 is highest consequence and can proceed independently of B02. B03's hold can precede both;
its final contents must remove couplings actually retired by B01. Packages are outcomes, not
fixed file or agent lanes. Coordinate edits to `library/admission.rs` and common bound consumers
with 57a and native arithmetic work.

<a id="b01"></a>

### B01 — Repair the cubic path with an admitted exact bracket owner

Source: [CBO-025 cubic direction](../library_leverage/reviews/mathematics-preparation.md#cbo-025).

**Inputs and decisions.** Consume finite evaluated coefficients, made exact as today, the
registered cubic acceleration, original residual/branch contract, cancellation and the original
operation deadline. Preserve degree-at-most-three and nondegenerate-polynomial refusal.
Before production replacement, settle two bounded design questions inside this package:

- **Stopping and conversion:** select a candidate rule serving existing residual/branch
  semantics. Compare exact rational-to-binary64 integer quotient/remainder conversion with
  directed multiprecision enclosures using the already enabled `float-mpfr` realization,
  `Float::from_rational_round` and `RoundingDirection::{Down,Up}`. The exact source entry is
  Numerica 3.0.1 `domains/float/multiprecision.rs`. A 53-bit intermediate followed by subnormal
  conversion is insufficient. Prefer the library enclosure route if its full conversion
  boundary can be verified without reimplementing a generic rounding kernel. Otherwise use a
  narrowly bounded exact integer conversion. Document the chosen candidate and progress rule
  before coding the refiner; it must not rely on the defective `Rational::to_f64` agreement.
  Correctly rounded *algebraic roots* remain an optional strengthening requiring a separately
  justified target; do not block the correctness fix on that stronger promise.
- **Work and storage admission:** derive a deterministic conservative step and integer-bit-size
  envelope from degree, coefficient heights and the isolated bracket, including square-free
  reduction, retained endpoints, temporary arithmetic, conversion and candidate vectors. Use
  existing work/byte allowances and typed refusal, not a new uncharged loop. Include roots at
  the subnormal scale: `x − 2⁻¹⁰⁷⁴` on `(0,1)` can need 1,074–1,075 bisections. A cap near 60,
  or an unproved log-width sketch, is not an established envelope. If the isolator has an
  unavoidable uncancellable region, preserve inter-call cancellation and explicitly retain its
  finite admitted envelope; do not claim per-step cancellation inside a library call.

**Mechanism and migration.** Form the square-free part once and isolate it once. Treat exact
singleton roots directly. For a nondegenerate interval, preserve the designated open-interior
root even if an endpoint is another root: use the sign immediately inside the interval,
including `u′(a)` when `u(a)=0`, then exact rational midpoint evaluation. Poll cancellation and
the original deadline each local step, before retaining output and during root verification.
A typed work/extent refusal replaces exhaustion; no partial root list may reach uniqueness
acceptance. Signed zero, underflow and finite overflow need an explicit conversion outcome.
Advance the supplier identity beyond the existing `symbolica.cubic-roots.v2` (proposed `v3`),
because completeness and returned points can change. Do not promise an ulp bound against the
old defective result.

**Deletion and consumers.** Move `real_points` entirely to this route and delete
`refine_root_interval`, its endpoint screen, its f64-agreement fallback trigger and the cubic
`IsolatedRoot`/`isolate_real_roots` path in the same tested change. Keep collocation's separate,
bounded polynomial-population `IsolatedRoot` use; no broad cache rewrite is authorized.
Original residual acceptance and selected reconstruction/IFT consumers remain the final domain
checks. Explicitly trace coefficient conversion in `library/admission.rs`
(`map_coeff(|c| c.re.to_f64())`): this is a related conversion risk, not proof that every
coefficient currently violates its promised accuracy. A decisive exact-reference comparison and
consumed-contract analysis must either integrate the correction and affected identity migration
here or record a reasoned bounded disposition in the coordinator before B01 closes. Do not
silently remove it from scope or impose a repository-wide correct-rounding contract.

**Targeted acceptance.** Extend `pse-math` units using independent exact factorization/sign
facts and an independently qualified conversion oracle, rather than the replaced ladder.
Exercise the containment cubic with branches selecting each distinct root, repeated-root value
versus regular-jet refusal, neighbouring endpoint roots, near-critical ±2⁻³⁰, signed powers of
two, exact midpoint ties, zero/subnormal transitions and finite overflow. Include cancellation,
deadline, deterministic work and byte refusal, and release/debug-equivalent outcomes. Verify
that repeated parameter changes do not enter the global root-cache API. Run
`just check-package pse-math` and targeted `just unit-package pse-math <filter>` with the
correctness route's force-validation closure; choose the filters from the resulting tests.
The retained containment and rounding-cell probes diagnose the old path, not the replacement.

<a id="b02"></a>

### B02 — Share checked dense decomposition without another stack owner

Source: [CBO-023](../library_leverage/reviews/mathematics-preparation.md#cbo-023).

**Target.** Extract the existing checked sequence into `pse-structural` over
`pounce_presolve::EqualityIncidence`: `hopcroft_karp` → `DulmageMendelsohnPartition` →
`SquareComponents` → `BlockTriangularForm`, validating dimensions, index ranges, matching and
partition consistency, coverage and prerequisite-first block order. Return the validated
blocks and needed inter-block edges. State the caller's qualified-stack premise explicitly;
the helper does not spawn or acquire another stack reservation. Apply `MATCHING_ROWS` refusal
and cancellation between stages at this common owner.

`incidence::analyze` keeps its semantic-ID mapping, dedicated charged stack and stable
lexicographical tie-breaking. `prepare_contract_inner` calls the same helper inside its already
charged math job, maps errors without losing contract/limit/cancellation meaning, requires a
complete matching and retains its source-restricted parameter-support propagation. Add the
internal dependency declaration once and verify its actual feature closure. Delete the two
separate pounce orchestration/check implementations once both callers have migrated; unrelated
compiler BFS/DFS and initialization SCC logic remain because they answer different questions.

**Acceptance.** `pse-structural` units attack invalid result coverage/order through an internal
validation seam and retain ordinary square/over/under cases and stable semantic ordering.
`pse-math` reconstruction units exercise a multi-block chain where a later unknown depends on
an earlier block's retained parameter, multiple residual branches and incomplete matching.
Check cancellation, too-large input and error mapping without allocating a huge recursive
fixture. Runtime controls establish that reconstruction uses the admitted job stack, not an
extra uncharged thread. Run package compile checks and targeted unit filters for both crates;
selected runtime reconstruction integration belongs to 57j after scope completion.

**Decision route.** The review proposes an architecture clarification at §15.3 and §7, rather
than a new algorithm contract. Operator confirmation and the normal architecture amendment
with revision row precede any rule-changing text. This draft does not amend those owners or
assert that the proposal is accepted.

<a id="b03"></a>

### B03 — Declare supplier coupling and consolidate residual mathematics bounds

Sources: [CBO-021](../library_leverage/reviews/mathematics-preparation.md#cbo-021) and
[mathematics CBO-001 input](../library_leverage/reviews/mathematics-preparation.md#cbo-001-math).

Add one Symbolica hold beside the exact workspace dependency, naming Dualizer expansion,
C++ export modeling, build-time abort retention and global root-cache premises that remain
after B01. Include Numerica `Rational::to_f64` double rounding and the consumed coefficient
conversion contract if B01 retains `map_coeff` conversion; record its correct-or-retain
disposition rather than assuming the cubic refiner repairs every conversion caller.
Keep the reason in the canonical Cargo pins declaration, routed through
[the dependency policy](../dev/dependency-policy.md#pin-reasons). The earlier `docs/pins.md`
pointer has no current file and does not justify a second hold declaration. Pin changes alone do not require an ADR. Retain
`CURRENT_ABORT`, `AbortBinding` and predict/admit/build/reconcile until a released per-call
abort and deterministic pre-allocation limit can replace them without weakening semantics.
When scoped cancellation changes elsewhere, preserve the plain atomic flag lent to callbacks.

Provide 57a's CBO-001 owner with actual B-tree consumers in structural flowsheet preparation,
compiler implicit construction, convexity construction and math retained extents. Adopt the
single conservative helper once its construction-versus-retention meaning is settled, including
keys/values and nested allocations without double charging. Remove the replaced local std node
geometry formulas. Keep the math-specific Symbolica expansion/export models, exact LDLᵀ work
accounting and residual-LM QR mirror with their actual owners; census/derive APIs cannot replace
pre-allocation prediction. Targeted tests compare supported nonempty/sparse/extreme extents,
checked arithmetic overflow, and admission-before-construction. Reconciliation tests must still
refuse actual products exceeding predicted allowances.

**Compiler-child migration.** Consume G03's working `pse-diagnostics::SupervisedChild` in
`native_compiled` rather than refactoring another local process owner. Preserve compiler park
on failed drain, original cancellation/deadline, unreaped leader/PGID reservation, private cache
copy and publication semantics. G03 supplies the mechanism; B03 moves this mathematics consumer
and deletes its replaced local supervision only with targeted interruption/descendant controls.

**CBO-024 bound-confirmation inquiry.** Separately trace `presolve.rs::tape` constant lowering
through `Constant::value()` and the use of FBBT tightened bounds. Determine whether tightened
bounds, not only infeasibility claims, are confirmed against the original authored facts after
nearest-rounded constant conversion. This is not B01's `map_coeff` conversion question. Record
the actual confirmation path and its independent controls, or a concrete gap and integrated
remedy retaining original-domain soundness. Use exact/inexact rational constants and cases where
a false tightened bound could exclude a valid point. Do not declare a defect or add duplicate
confirmation when existing evidence settles it. B03's inquiry closes with this explicit
correct-or-retain disposition; the parallel-IR premise remains rejected.

**Conditional U1 branch.** Prepare a technical option for Symbolica per-call abort, deterministic
pre-allocation limits on `vectorize`, deterministic checkpoints in `derivative`/`replace`/
`coefficient_list`, containment-aware isolation/refinement and bounded or non-caching root
isolation. No contact occurs under this plan-writing authority. Adoption requires an inspected
release plus local cancellation, admission, conversion and completeness controls. Only then
retire the corresponding local mirror/binding; keep inexpensive post-build reconciliation.

## Retained and deferred scope

- **CBO-022:** retain Salsa generation-local semantics, process `ModelingBodyRetention`,
  body-local `ScalarCache` products and the ADR-0212 cross-process compiled-module cache.
  They have distinct lifetimes and keys. Host process/publication changes in other companions
  migrate the compiled cache as a consumer if needed; no cache unification is scheduled here.
  The low-value public Salsa label/`AsId`/`current_revision` lead remains conditional: replacing
  generated ingredient access with event-side label/ID maps trades compile-time supplier drift
  for runtime mapping. Keep current plumbing until a released public surface covers all needed
  semantics and removes real maintenance without another cache or map authority.
- **CBO-024:** retain the project factorable projection and library-format consumers. Operator
  tables encode distinct semantics; no universal intermediate language is scheduled.
- **CBO-025 exact LDLᵀ:** retain exact rational semidefinite/Gram-factor certification,
  deterministic work exhaustion as Inconclusive, cancellation and cone consumers. Floating
  faer, SPEX zero-diagonal refusal and scalar big-number libraries do not supply that complete
  contract. This is a reasoned retention, not uncompleted replacement work.
- **CBO-023 growth:** raising supported case limits beyond 100,000 rows triggers comparison of
  an iterative pounce matcher with SuiteSparse `btf_maxtrans`/`cs_dmperm`. A maintainer-owned
  fork is local integration; an external upstream option follows U1's separate contact boundary.
  Preserve matching order and typed refusal. Changing the algorithm does not by itself release
  the shared stack policy in KINSOL, native child and runtime owners.

## Execution checkpoint — 2026-10-27

B01 is integrated in `74cc0636c`: maintained bounded supplier entry points admit work/storage
before square-free reduction and isolation, preserve root associations and enumerate the chosen
midpoint/endpoint candidates through certified rational conversion. An unrepresentable root
refuses the complete result instead of silently dropping that association. Twelve selected
cubic/conversion/collocation controls passed in both debug and release. The broader evaluator coefficient conversion deliberately retains its existing numerical
realization: the actual supplier `map_coeff` control maps `(2^53+3)/(2^53+1)` to `1+2 ulp`,
where certified nearest is `1+1 ulp`. No general RNE promise exists at that backend boundary.
The three production conversion paths and their identities remain unchanged; exact proof
consumers enclose both authored and current numerical values. The canonical hold names the
remaining double-rounding coupling and requires revisiting a certified-rounding consumer or
supplier/API change. The bounded root endpoint normalization does not change collocation's
unbounded isolation route or its separate identity.

B02 is integrated in `3b12321fd`: case incidence and implicit reconstruction use the same checked
non-spawning matching/DM/BTF owner. Their caller-qualified stack remains charged; no second
scheduler or matching thread was introduced. Selected structural and reconstruction controls
passed. The actual admitted runtime preparation journey remains for J.

B03 has canonical source-coupling holds in Cargo metadata and the shared allocation bounds.
The minimum G03 compiler/capture supervisor prerequisite is integrated in `c89aa0e79`; 57g owns
its remaining consumers. Its later independent-review hardening for descriptor-proven exited processes with unavailable
namespace metadata is integrated in `b2cbd8576`, with eighteen diagnostics controls passing in
debug and release. This host retained ordinary zombie PID-namespace links: the missing-metadata
fixtures are explicitly injected, not a reproduced ordinary fast-exit hang. Actual compiler/
capture refresh remains with J. Exact-source FBBT inquiry found that rounded point constants and
rounded exponent classification can make the compiler's big-M enclosure unsound. The selected
remedy extends the existing supplier FBBT leaf with an interval constant, uses exact exponent
predicates, and adds independent cancellation/amplification controls. This is integrated in `73afd65b0`, with thirteen math and three compiler derived-M/rebind
controls passing in both debug and release. Disabling backend bound tightening did not settle
the compiler consumer; the independent authored cancellation regression now reaches it. No parallel IR or
mandatory new coefficient-rounding contract is selected.

## Verification and evidence limits

**Interface-checked:** current decisive owners and installed Symbolica/Numerica 3.0.1 APIs were
read during authoring. Context7 was consulted first for Symbolica; it supplied general setup,
not the exact refinement/conversion contracts, so pinned installed source and the review's
source references govern these choices. The inherited containment, refinement and rounding-cell
probe records remain *Tested* only under their recorded conditions.

**Proposed:** all packages and replacement mechanisms. **Not run:** product builds, probes,
solver/provider workloads, targeted tests and broad qualification during this documentation task.
57j owns selected assembled scientific/Python journeys and scope-end checks after implementation.
No performance or whole-subsystem correctness claim follows from this foundation assessment.
