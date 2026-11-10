---
title: "Plan73d: Exact structure and numerical assurance"
status: in-progress
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73d-exact-structure-and-numerical-assurance.md
doc_topics: [current-work, design, libraries, numerical-execution, validation]
review_sources:
  - docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md
  - docs/functional_reviews/symbolica-ecosystem/exact-structure.md
  - docs/functional_reviews/symbolica-ecosystem/numerical-realization.md
---

# Exact structure and numerical assurance

## Owned outcome and baseline

This companion of [Plan73](73-symbolica-ecosystem-improvements.md) owns
O04 and O05: a selected, source-linked exact-block diagnostic and a numerical diagnostic
whose claim states what the arithmetic establishes. It owns the changes to current exact
extraction, coefficient/class-evidence consumers and assurance readers needed for those
outcomes, including deletion of replaced routes. The entire new series completes before
Plan66 resumes. Existing Plan66 packages are context for overlaps, never prerequisites or
an alternative owner of this work's structural migrations and acceptance.

The inspected planning baseline is `e9f4e7b7`. The source review records its earlier
`7d89d16f` evidence separately. **Interface-checked:** the maintained Symbolica 3.0.1
matrix/evaluator APIs and installed Numerica 3.0.1 domain contracts described below.
**Proposed:** every changed operation, package, migration and verification in this plan.
No implementation, compile check, test or performance experiment was performed in authoring.

The relevant foundations already exist: `class_evidence.rs` aggregates admitted exact
expressions and extracts selected sparse affine rows; `coefficients.rs` already shares
some request-local objectives and projects exact polynomial structure; `rational_binary64.rs`
provides directed exact conversion; `implicit_cubic.rs` uses maintained bounded roots;
`convexity.rs` supplies symmetric-pivot LDLᵀ/Gram witnesses. This plan strengthens their
composition and adds useful operations; it does not describe those existing delegations as
missing. Current scientific authority remains blueprint §§7, 14, 16, 23, with changes routed
through the coordinator's decision packages before a new durable operation/kernel contract.

## What changes, what is consolidated, and what is new

The result is two new developer operations over admitted science: parameter-aware exact-block
inspection and claim-scoped numerical diagnosis. The work also changes existing coefficient
classification/projection, rather than delivering those operations beside unchanged repeated
extraction. These are separate obligations:

| Work | Current pathway and resulting change | Owning package |
| --- | --- | --- |
| Consolidate existing exact work | `shape_facts_with_objectives` already passes `ExactObjectives` into `project_coefficients`. Replace the remaining objective derivative-classification followed by separate polynomial expansion with a compatible request-owned exact objective view consumed by both; preserve sparse row proof. | [D02](#d02) |
| Change a numerical projection policy | `coefficients::number` currently uses general scalar evaluation; exact rational coefficient projections will use the existing certified nearest-even boundary with their existing nonzero-underflow refusal. Exact rational conic projection in `curvature.rs` receives the same explicit range policy. | [D04](#d04) |
| Add exact diagnostic capability | Select original rows, unknowns and symbolic parameters, then return rank/consistency/exceptional conditions with original-row attribution; bounded reduction is a separate decision. | [D01](#d01), [D03](#d03) |
| Add approximate and enclosure capabilities | Chosen precision/error estimates and a restricted supplier ball evaluator produce distinct claims through one request/result path. | [D04](#d04), [D05](#d05) |
| Cross-cut result consumption and preserve certificates | Extend the protected original-result journey for these products, preserve consumed LDLᵀ/Gram/root evidence, and finish each explicitly bounded optional decision. | [D06](#d06), [E04](73e-analytical-products-and-native-presentation.md#e04) |

The actual baseline deletions are the replaced objective classification/projection passes and
the selected rational conversion branches, after all their callers move. There is no claimed
existing exact-block diagnostic, generic interval interpreter or duplicate certified conversion
engine to delete. New capability packages state that distinction below. A mathematical consumer
is an acceptance journey, not a restriction against a coherent replacement across several owners.

## Target and focused foundation decisions

### Original product, selected demand and exact views

A request selects admitted equations/outputs, unknown coordinates, symbolic parameters,
physical/source context and the desired product. The mathematical owner derives compatible
views from the same original resolved product and its actual guards, definitions and provider
premises. The shared admitted/mapped-product contract belongs to companion A; this document
owns the exact views and the two current consumer migrations. Request-local ownership avoids
a process-global CAS cache or a second scientific identity scheme.

Separate a sparse evidence request from a bounded dense parameter-aware block. Selected
sparse rows continue using derivative/zero-substitution extraction where that is the fitting
operation. Polynomial/coefficient products can be retained once when class evidence,
coefficient projection and exact inspection consume the same selected expressions, coordinate
support and lifetime. Never build a dense matrix over every model variable merely to unify
representations. A wide model with a one-row demand must examine only its admitted support;
shared preparation is an aid to that behavior, not permission to flatten the model.

The baseline already has request-local sharing through `ExactObjectives` and
`shape_facts_with_objectives`; extend that owner rather than introduce a second cache. In
`class_evidence::class_evidence` and `CasePlan::coefficients`, aggregated objective atoms are
already handed to `project_coefficients`; affine row proofs are already consumed from `Facts`.
The residual duplicate is objective structure: classification takes first/second derivatives
over the objective columns, then projection calls `expand().to_polynomial_in_vars::<u16>` on
the same objectives. In addition, `coefficients_with_facts` has no retained objective product
and its projection deliberately calls `aggregate_objectives` again. These are different from
the selected-row derivative proof, which must remain demand-owned. Actual
reference consumers are [class evidence](../../crates/pse-math/src/class_evidence.rs) and
[coefficient projection](../../crates/pse-math/src/coefficients.rs). Retain their current
original-domain and consumed-value matching. Changed fixed parameters, provider premises,
coordinate layout or selected output support invalidate the affected derived view; binding
new numerical inputs to compatible admitted structure does not identify the scientific results.

### Parameter-aware exact diagnostics

Select a small affine block diagnostic, with explicit row/unknown and expression/term limits.
It returns coefficient/righthand-side mathematics, rank/consistency at requested exact parameter
specializations, a generic result only with stated exclusions, and source-linked redundant or
inconsistent rows where the native reduction establishes them. A bounded or interrupted
operation reports incomplete/inconclusive scope; it cannot classify unexamined specializations.
This explains accounting redundancy or degeneracy without pretending algebraic rank is the
model's structural matching result.

Use `AtomCore::system_to_matrix` to extract a
`Matrix<RationalPolynomialField<Z,E>>` and its right-hand side, then native
`Matrix::{row_reduce,rank,solve}` as appropriate. Symbolica retains parameter-dependent
rational-polynomial coefficients rather than substituting every parameter into Q.
References: maintained Symbolica 3.0.1 [public conversion](../../vendor/symbolica/src/atom/core.rs)
and [conversion implementation](../../vendor/symbolica/src/solve.rs); Numerica 3.0.1
[Matrix API](https://docs.rs/numerica/3.0.1/numerica/tensors/matrix/struct.Matrix.html)
and [domain qualification](../library_leverage/libraries/numerica/domains-and-certification.md).
`solve` distinguishes inconsistency from underdetermination; `solve_any` chooses zero free
coordinates and does not supply a canonical solution family or minimum-norm answer.

Generic elimination can divide by parameter polynomials. Record those nonzero conditions
and original denominators, or restrict the result to individually evaluated exact
specializations when available native output does not establish generic coverage. If the
native rank/reduction operation does not expose the conditions needed for an honest generic
claim, add a maintained supplier condition/witness projection rather than reconstructing a
second elimination engine in product code. Specialized checks re-extract/reduce the original
selected block at exceptional values; substituting into a generic inverse is insufficient.

The revealing accounting example is `x+y=1`, `x+p*y=q`. At `p=2,q=3` the solution is
`x=-1,y=2`; at `p=1,q=1` the block is underdetermined; at `p=1,q=2` it is inconsistent.
Generic uniqueness excludes `p=1`. A physical nonnegativity requirement can still reject the
first algebraic solution: exact algebra does not grant physical feasibility. Include an
original canceled denominator case to preserve source-domain obligations.

Selected small-block elimination is a conditional suboperation, decided in D03. Enable it
only when coordinate recovery, exceptional conditions, original residual/domain validation
and bounded supplier work all compose for this named block. Otherwise deliver the selected
diagnostic and record the reduction disposition and observable reopen condition. Broad
symbolic presolve, Gröbner/resultant workflows, modular CRT/reconstruction and fraction-free
replacements are not required for this first diagnostic. Investigate them only if this actual
block exhibits consequential coefficient swell and a complete native route is preferable;
no custom modular execution engine is implied.

### Numerical claim vocabulary and realization

One diagnostic request states exact input points or input intervals, selected precision,
admitted output/derivative demand and requested claim. Its result keeps these meanings distinct:

| Claim | Meaning and limit |
| --- | --- |
| Ordinary result | Existing binary64 numerical realization; no new accuracy certificate. |
| Chosen-precision estimate | `DoubleFloat` or explicitly chosen multiprecision estimate, including actual provider precision/support. More digits alone establish no containment. |
| Propagated-error estimate | An `ErrorPropagatingFloat` estimate with its input and arithmetic assumptions; not a rigorous enclosure. |
| Exact identity | Exact admitted algebra with explicit original domain and parameter conditions, not empirical property validity. |
| Validated enclosure | Directed supported arithmetic encloses the requested input set; unsupported, whole/unbounded or inconclusive outcomes remain visible. |

Native `ExpressionEvaluator::map_coeff_with_prec` composes exact retained preparation with
`DoubleFloat`, `Float` or `ErrorPropagatingFloat` realizations, followed by `try_evaluate`.
Use explicit precision for variable-precision domains and external constants. The maintained
3.0.1 [domain implementations](../../vendor/symbolica/src/evaluate/domain.rs) and
[evaluator mapping/execution](../../vendor/symbolica/src/evaluate/evaluator.rs) are the relied-on
contracts. Mapping has panic paths for unavailable constants, so admission and a fallible
supplier projection must establish supported constants/functions before mapping. No panic,
ordinary scalar callback or conversion fallback may be mistaken for an eligible diagnostic.
Supplier callbacks are admitted at their actual precision; an unavailable realization refuses
that claim rather than reporting the requested precision for a binary64 callback.

Exact-to-binary64 projection continues through the single
[rational conversion owner](../../crates/pse-math/src/rational_binary64.rs), which uses
`Float::from_rational_round` and nearest-even boundary agreement. This does not authorize a
silent change to every ordinary `map_coeff` conversion: migrate only consumers requiring this
certified boundary, identify their changed rounding behavior, and remove their duplicate
converters. Approximate rational explanation candidates retain changed-value status. A simple
rational inside an enclosure is not an identity.

### Selected restricted ball-evaluator bridge

The inspected Symbolica 3.0.1 `EvaluationDomain` implementations contain no `RealBall`
implementation. Mapping an evaluator directly to balls is therefore an implementation task,
not a capability already supplied by the trait. D05 owns a maintained supplier extension:
a restricted checked arithmetic evaluator backed by Numerica `RealBall`, with the
`EvaluationDomain` bridge inside Symbolica where the trait can lawfully be implemented.
Use one supplier interpreter and native arithmetic; do not create an application tree walker.
A raw generic trait implementation alone is insufficient because inherited `Real` methods
include uncertified transcendental operations.

The checked construction path accepts only real exact rational constants, input balls,
addition/subtraction/negation, multiplication, division/inversion and integer powers composed
from those operations. It validates the actual final program as well as source eligibility.
Every branch, comparison-dependent selection, noninteger power, transcendental, external
callback or rounded external constant is rejected for this first enclosure profile. Rational
constants and rational endpoint inputs use `RealBall::from_rational_bounds`; binary64 endpoints
retain their exact dyadic meaning. Validate ordering/finiteness before construction.
Negative powers/division with possible zero denominators produce an explicit unbounded or
inconclusive outcome, never a finite validated answer. Source-domain admission still precedes
simplification and interval propagation.

The certification basis is Numerica 3.0.1
[`RealBall`](https://docs.rs/numerica/3.0.1/numerica/domains/float/struct.RealBall.html),
whose source `domains/float/interval.rs` certifies basic directed arithmetic; its integer-power
implementation composes multiplication, and inverse detects zero-containing balls. See the
[exact-version contract account](../library_leverage/libraries/numerica/domains-and-certification.md).
`Float::{from_rational_round,add_round,sub_round,mul_round,div_round}` supplies directed
construction/arithmetic, not a general transcendental theorem. The bridge's version, checked
program profile and supplier provenance join the result's assurance premises. Complex balls
remain a separately triggered extension for an actual complex-valued diagnostic.

### Preserve stronger specialized evidence

Keep symmetric-pivot exact LDLᵀ/Gram and negative-direction witnesses in
[convexity](../../crates/pse-math/src/convexity.rs). A rank test/general solve cannot replace the
consumed certificate. `RingOps::sub_mul_assign` may consolidate product/subtraction, but the
general rational implementation is not a fused-performance guarantee; adopt it only if it
simplifies the existing algorithm while preserving witness and cancellation semantics.

Keep the maintained bounded square-free/isolation route and admitted midpoint refinement in
[implicit cubic](../../crates/pse-math/src/implicit_cubic.rs), with supplier provenance recorded
in [bounded-root maintenance](../../vendor/symbolica-bounded-roots.md).
`Rational::round_in_interval` can return an endpoint and does not guarantee halving. An
alternative interior-search refiner is conditional on an actual consumer and a complete
contraction/limb/cancellation argument; no retry ladder replaces the current route merely to
use the API. Approximate `nsolve` is a separate research estimate, never selected-root proof.
Native library multiplication/exact division already delegate arithmetic; do not wrap those
kernels again without an actual missed composition.

## Packages

The coordinator owns combined opportunity disposition and series completion. This table is
the sole progress owner for these packages. Every package starts open; no existing acceptance
or old-plan status is transferred. Dependencies below are capabilities, not whole-document
barriers. Supplier changes coordinate with companion B's provenance and maintained-source
contract; they do not wait for an unrelated old supplier package.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [D01](#d01) | paused | Coordinator decision route; A's admitted original-product and binding contract | Exact/assurance request, claim and refusal semantics implemented with source/domain attribution. |
| [D02](#d02) | paused | D01; A's working selected-product demand/lifetime | Class evidence and coefficient projection consume compatible shared exact views; replaced repeated extraction deleted; sparse demand preserved. |
| [D03](#d03) | paused | D01–D02; native matrix condition/provenance support | Selected parameter-block diagnostic delivered at ordinary/exceptional values; reduction investigation disposition and any selected implementation complete. |
| [D04](#d04) | paused | D01; A's working exact evaluator product; B's supplier-qualified mapping/error contract | Chosen-precision and propagated-error diagnostics integrated without stronger false claims; certified conversion consumers migrated. |
| [D05](#d05) | paused | D01; D04's claim/result path; checked supplier bridge and provenance | Real supported arithmetic enclosure executable through native evaluator; unsupported program and singular interval outcomes explicit. |
| [D06](#d06) | paused | D02–D05; E's protected presentation consumer when exposing mathematical output | Witness/root/conversion preservation, conditional dispositions, original-result readers and all in-scope replacement deletions accepted. |

<a id="d01"></a>
### D01 — Admit exact and assurance requests

The operation receives original selected expressions/rows, unknown and parameter coordinates,
fixed values or intervals, requested precision and claim. It returns examined scope, original
source/physical premises, actual claim and typed unsupported/partial outcomes. Admission retains
original guards before algebraic cancellation. It cannot accept duplicate/absent unknowns or
pretend an unsupported provider has the requested precision.

**Footprint and contracts.** Extend the scientific boundary in
`pse-compiler/src/workspace/modeling/executable.rs` and `typed_math`, the exact/diagnostic API in
`pse-math`, and orchestration in `pse-runtime/src/workflow`. Add authored registry declarations
for the actual operation/result in `pse-schema/src/catalog/native_math.rs` and the boundary
projection owners, and regenerate; generated files are never edited. Symbolica
3.0.1's selected `AtomCore` operations and Numerica 3.0.1 arithmetic are consumed through the
contracts at D03–D05, without exposing raw Atoms as another authoring authority. The independent
native host retains command/process custody only.

**Readiness and migration.** Begin once A issues a selected original product with complete
binding/domain attribution and the coordinator has routed the new operation/result contract.
E can consume this early contract without waiting for all D realizations. These exact and
assurance operation variants are new; no predecessor diagnostic implementation is identified
for deletion. Migrate their producers and scientific entry points together rather than exposing
an isolated library demo.

**Independent acceptance.** Exercise the actual operation with a selected law, unsupported
provider/branch, duplicate or absent unknowns, original denominator violation hidden by
cancellation, interruption and changed source. Assert declared scope and physical/source IDs
from the authored request independently of returned mathematical text. A retained old result
must continue to identify its old premises.

<a id="d02"></a>
### D02 — Share exact extraction at actual consumers

Extend `class_evidence::ExactObjectives` into the request-owned selected objective product. It
carries aggregated exact atoms, actual coordinate maps, consumed fixed-value assumptions and a
lazily constructed sparse polynomial view where supported. Classification reads degree/support
and quadratic structure from this view; `project_coefficients` reads its terms to assemble the
existing numerical `Coefficients`. Native Symbolica 3.0.1 `AtomCore::expand`,
`to_polynomial_in_vars::<u16>` and polynomial coefficient/exponent access already used in
[coefficients.rs](../../crates/pse-math/src/coefficients.rs) own the algebra. Unsupported
polynomial representation stays distinct from a mathematical negative; retain meaningful
nonpolynomial derivative witnesses without running a second polynomial objective classifier.
Original-domain admission and continuing proof allowances still govern both consumers.

**Actual cutover.** Migrate `CasePlan::{class_evidence,coefficients,coefficients_with_facts}` and
the internal `shape_facts_with_objectives` / `coefficient_projection` composition. The existing
combined request already shares objective atoms. Replace its remaining polynomial-eligible
objective first/second-derivative classification followed by independent polynomial expansion
with one shared view. Carry that view through operations that request facts and projection
separately within the same compatible lifetime, removing their now-unnecessary
`aggregate_objectives` call. A genuinely new facts-only request may prepare its demanded
objective view anew through the same owner; never serialize supplier atoms into `Facts` or
invent a global cache.

Selected-row `affine_row_evidence` / `prove_affine` retain sparse derivative/zero-substitution
proof. Projection already reads established row entries from `Facts`; there is no duplicate row
polynomial extractor to remove. Keep term/byte/refusal accounting across atom and polynomial
lifetimes and release views at their last consumer. The footprint is
`pse-math/src/{class_evidence,coefficients}.rs`, compiler callers composing the joint request,
and D03's new exact-view consumer. Delete replaced objective loops, transfer-only plumbing and
mechanism-specific fixtures as soon as the migrated target tests pass; keep independent behavior
checks. Readiness is A's selected demand/lifetime and D01's assumption identity, not completion
of D03 or an unrelated whole-model representation.

**Independent acceptance.** Both current consumers must consume the compatible view for a
quadratic primary objective, linear later objective and cancelling contributions. Use manually
specified exact coefficients and Hessian values. Include nonlinear later-objective and
nonpolynomial witnesses, mismatched fixed values/guards, a separately requested facts-based
projection, and a wide model with one selected sparse row. The latter must not allocate or
classify all objective columns merely to prove that row. Supplier self-comparison is insufficient.

<a id="d03"></a>
### D03 — Deliver parameter diagnostics and settle reduction

The selected operation extracts original named rows into coefficient/righthand-side mathematics,
reports exact specialization rank/consistency and, where established, generic exclusions and
source-linked dependencies. Use maintained Symbolica 3.0.1 `AtomCore::system_to_matrix` and
Numerica 3.0.1 `Matrix::{row_reduce,rank,solve}` under the contracts above. Add a supplier
condition/witness projection if needed; otherwise explicitly withhold unsupported generic claims
while still delivering exact specialization results. Generic inversion followed by substitution
is not an exceptional-case diagnostic.

**Footprint and readiness.** Add the block operation beside D02's exact views in `pse-math`,
compiler row/parameter selection and D01's runtime result producer. Supplier condition support,
if needed, belongs in `vendor/symbolica/src/solve.rs` and maintained-source notes coordinated
with B. Begin when original selected expressions/maps are available; dense extraction is only
for this bounded block. This is a new diagnostic: no current product elimination is alleged
for deletion.

**Bounded reduction decision.** On the accounting block `x+y=1`, `x+p*y=q`, determine whether
a reduced evaluation product preserves coordinate recovery, original residual/domain checks,
exceptional conditions and bounded/cancellable supplier work. If yes, deliver the reduced
product and its actual diagnostic evaluation consumer, removing that path's replaced local
preparation while retaining original residual validation. If no, retain the unreduced diagnostic
and record the failed criterion plus the concrete contract or workload change that would reopen
reduction. Do not add an unused general presolver.

**Independent acceptance.** Check `p=2,q=3` gives `x=-1,y=2`, `p=1,q=1` is underdetermined and
`p=1,q=2` inconsistent; generic uniqueness excludes `p=1`. Verify expected ranks directly from
these small exact matrices. Add an original denominator violation, nonlinear extraction refusal,
physical nonnegativity rejecting the first algebraic solution, selected-row limits and
cancellation. Row attribution must follow original identities through reduction. Structural
matching, solver selection and whole-model presolve retain their distinct owners.

<a id="d04"></a>
### D04 — Deliver approximate diagnostics and explicit rational projection policies

Map A's admitted exact evaluator once per requested `DoubleFloat`, chosen-precision `Float` or
`ErrorPropagatingFloat` realization through Symbolica 3.0.1
`ExpressionEvaluator::map_coeff_with_prec` / `try_evaluate`. Preserve exact constants until
mapping, bind inputs without rebuilding source preparation, retain actual provider precision and
release mapped workspaces at operation settlement. Add fallible supplier mapping where current
constant/function mapping can panic; unsupported support is a typed refusal, never a scalar
callback fallback claiming higher precision. The existing supplier files
`evaluate/{domain.rs,evaluator.rs}` and `pse-math/src/library/admission.rs` bound this change;
B coordinates maintained-source provenance.

The conversion migration has concrete clients and deliberately different policies:

| Client | Current route | Target and removal |
| --- | --- | --- |
| `coefficients::number`, also called by `class_evidence::Allowance::number` | General `Atom::evaluate` into `f64`; finite/nonzero-underflow checks follow. | For exact real rational constants use `rational_binary64::nearest`, keeping cancellation, finiteness and nonzero-underflow refusal. Remove the general-evaluation branch for that rational case. Nonrational constants retain an explicitly approximate supported evaluation or representation refusal, never a guessed rational. There is one existing helper, not two duplicate converters. |
| `curvature.rs`'s `Shape::Quadratic` conic projection | `q.constant`, linear coefficients, Gram weights and row terms call `to_f64`; zero terms are then removed. | Route exact rational-to-binary64 projections through the same certified boundary with explicit range refusal before nonzero terms can disappear. Preserve native square-root/rounding meaning for the subsequent numerical cone construction; this does not make the numerical cone an exact witness. Remove those replaced `to_f64` branches and propagate refusal through the actual conic projection result. |
| New exact/assurance diagnostic producer | No baseline route. | Use `nearest` for requested binary64 values and `enclosure` for outward exact-rational endpoints, retaining their distinct meanings in D01's result. No predecessor to delete. |
| Existing `implicit_cubic` and `factorable::Constant::enclosure` | Already use `rational_binary64::{nearest,enclosure}`. | Preserve and reuse; there is no alternate certified boundary to retire. |

The certified boundary is [rational_binary64.rs](../../crates/pse-math/src/rational_binary64.rs),
using Numerica 3.0.1 MPFR `Float::from_rational_round` and Rug 1.30.0 nearest-even projection.
Ordinary `map_coeff(...to_f64())` in interpreted/compiled solver preparation, ordinary
`factorable::Constant::value`, and collocation are distinct numerical-policy clients; they are
not silently changed by this diagnostic package. The inspected source contains those routes,
so a statement that all rational conversion is already unified would be false. Broadening their
policy requires an explicit consumed behavior/qualification decision, rather than an invented
count of duplicate engines.

**Readiness and acceptance.** D01's result contract, A's working exact evaluator and B's fallible
mapping contract are sufficient; D03 is independent. Check a cancellation-sensitive relation
against exact arithmetic, rational ties and adjacent halfway cases, signed zero, subnormal and
overflow boundaries, and a provider with only binary64 support. Exercise coefficient and conic
consumers with a tiny nonzero rational that would vanish after ordinary conversion; assert
explicit refusal rather than erased support. Verify coefficient/Hessian and Gram witness
meaning independently. Extra digits and propagated-error estimates never populate enclosure
fields. Delete only the actual branches named above and their mechanism-only tests after all
consumers move.

<a id="d05"></a>
### D05 — Implement and qualify the restricted enclosure bridge

Add the maintained supplier `EvaluationDomain` bridge and checked arithmetic-program constructor
specified above, composing native evaluator instructions with Numerica 3.0.1 `RealBall` basic
arithmetic and `RealBall::from_rational_bounds`. The final program admits only the rational,
addition/subtraction/negation, multiplication/division and integer-power profile. A generic trait
implementation alone does not certify inherited transcendental methods. Check both source
eligibility and final instructions so normalization cannot hide an unsupported source obligation.
The diagnostic returns finite enclosure, unbounded/singular, unsupported or incomplete scope
under D01's claim vocabulary.

**Footprint and readiness.** Supplier `evaluate/{domain.rs,evaluator.rs}` owns checked execution;
`pse-math` admission owns scientific source eligibility and D01's runtime operation publishes
results. Begin after D04's mapping/refusal/result composition works and B can record the checked
bridge revision and provenance. Coordinate shared supplier edits. This is a new capability:
there is no baseline application interval evaluator to delete, and no new application tree
walker is allowed. Do not replace existing transcendental FBBT or specialized root containment.

**Independent acceptance.** Check exact rational polynomial containment against analytically
known extrema, distinguish input intervals from arithmetic uncertainty, and test nonzero and
zero-crossing denominators. A forbidden operation erased by simplification must still fail
source eligibility; unsupported surviving instructions must fail the supplier constructor.
`sin`, `log`, noninteger powers, branch selection and external callbacks refuse enclosure even
when approximate evaluation works. A negative power crossing zero cannot report a finite ball.
Include interruption with no success-shaped stale output. The independent expected interval
must not be generated by the same ball evaluator.

<a id="d06"></a>
### D06 — Complete consumed certificates, readers and bounded decisions

Keep `convexity.rs`'s symmetric-pivot LDLᵀ/Gram and negative-direction witnesses and
`implicit_cubic.rs`'s maintained square-free/isolation/midpoint-refinement certificate. General
rank/solve is not their replacement. Carry D01/D03/D04/D05 products into E04's protected
original-result publication/read path, retaining attempt, manifest, original physical closure
and actual claim without recomputing mathematics while reading. The footprint includes these
`pse-math` consumers, `pse-runtime/src/workflow/{durable,result_projection,result_inspection}.rs`
and E's new stored mathematical projection. Readiness is the delivered D products and E04's
stored-result contract, not the whole E05 optional investigation.

The remaining leads close against named questions; none creates an open-ended framework:

| Lead | Actual case, question and decision consequence |
| --- | --- |
| Fraction-free/modular/CRT/GCD alternatives | Only D03's accounting block, or its selected bounded larger exact-block fixture if it exposes consequential intermediate growth. Does native `solve_fraction_free` or a higher-level native polynomial/elimination route preserve all exclusions and original recovery with less repeated expansion? Adopt the complete native route and delete its replaced local path if yes. Otherwise retain ordinary exact elimination; reopen on that actual block's unsupported domain/growth, not on library availability. No separate benchmark or custom modular engine is mandatory. |
| `RingOps::sub_mul_assign` | Inspect the actual LDLᵀ residual update in `convexity.rs`: can the native operation simplify that update without changing witness/cancellation semantics? Adopt and delete the replaced arithmetic helper only if useful. Otherwise retain it; general rational dispatch is not a fused-speed promise. |
| Rational interior search | For `implicit_cubic`'s admitted midpoint refinement, does `Rational::round_in_interval` supply an interior point with the required contraction/limb/cancellation argument? Its endpoint possibility currently prevents that inference. Retain the existing route unless the complete argument/contract is established, then replace and delete the midpoint-only mechanism. Reopen on an actual selected-root refinement need or stronger supplier contract. |
| Complex balls | Does any selected D01 diagnostic have complex inputs/outputs? The first real arithmetic profile does not. Record not selected unless such a request is admitted with an exact basic-operation contract; adoption includes constructor, output/claim and independent complex arithmetic checks. A real-ball trait implementation is insufficient evidence. |
| Approximate `nsolve` | Does D03's accounting diagnostic need a nonlinear initial estimate? It does not. Record not selected for the affine case; reopen for a named nonlinear diagnostic with residual/scaling/domain/cancellation semantics. Native step-size convergence alone never becomes selected-root proof. |
| Rational explanations | On D04's cancellation/rounding result, would `truncate_denominator`, `round` or `round_in_interval` help explain the actual value/interval? If selected, return a separate changed-value candidate and exact difference/containment status, tested independently; otherwise record no explanatory demand. Never populate exact-identity fields from an interval member. |

**Cutover and independent acceptance.** Preserve independent LDLᵀ reconstruction/negative-direction,
root containment/original residual, and rounding-boundary checks at the changed consumers. Read a
stored D result after source evolution with science construction unavailable, using E04's
permission and manifest checks. Verify that incomplete readback never changes the stored
operation's completion. Each lead records evidence, adopted-and-delivered or not-selected status,
failed criterion and observable reopen condition here; a favorable decision includes its actual
producer/consumer and deletion. Delete only helpers truly superseded by adopted routes, plus
D02/D04's identified replacements; no legacy certificate code is removed merely because a new
matrix API exists. New evidence/kernel contracts follow the coordinator's decision route.

## Verification and completion

**Proposed:** during execution use `scripts/pse check <affected-package> --all-targets` and
focused `scripts/pse test <package> --lib -E 'test(=NAME)'` selections for each meaningful
behavior above; integration targets use `<package>:<target>`. Determine actual test names from
the implemented target rather than treating illustrative cases as existing tests. Arrow-consuming
correctness closures explicitly use force-validation through the supported route. Baseline is
zero failures; no command in this paragraph has been run for plan authoring.

D closes only with selected exact/assurance operations reachable through actual product consumers,
source-domain and claim refusals, two migrated exact consumers, preserved specialized witnesses,
protected original-result readback and deleted replacements. Package-local evidence establishes
its tested scope; assembled workflows and scope-end hygiene/governance belong to companion H
and the coordinator. Do not run integrated/static campaigns per package or add improvement
benchmarks by default. No runtime, memory or speed improvement is claimed as Measured.

At closure record evidence labels, command/mode/scope/zero baseline and actual failure count,
material remaining limits, one mistake corrected and deliberate deviations. Move enduring
contracts to their architecture owners through the decision route, then retire this plan with
the series under ADR-0096. There is no implementation Outcome yet.

## Execution checkpoint — 2026-11-10

Execution begins at clean baseline `e427c474b`. ADR-0265/0266 establish the new scientific contracts before dependent implementation. The approved scope includes typed selected-observation members beside ordinary case solves. Existing A–C products remain foundations; root owns shared schema/native/result integration and three assignments own D, E and F implementation. Package completion requires actual consumer cutover, deletions and scoped acceptance; Plan66 and H retain their separate scope.

The shared exact-objective and controlled diagnostic core is committed in `c0736fa18`.
The original selected core batch passed 36 tests; the second diagnostic/Gram/root batch passed
16 tests. Compiler source admission, retained reduction use, final supplier controls and the
ordinary native mathematical publication/reader are still being integrated and qualified.
These bounded passes do not close D01–D06 or qualify physical feasibility.

## Paused handoff — 2026-11-10

The maintainer requested cancellation and a handoff. D01–D06 remain unclosed and execution
is paused; no further implementation or qualification is running. The implementation is
preserved through `4aa7293a1` and the shared integration commits. Paused package status does
not mean the delivered mathematics is absent.

**Implemented:** `ExactObjective` views now feed class evidence and coefficient extraction,
replacing eligible objective derivative classification followed by repeated polynomial
expansion. Sparse affine proofs and nonpolynomial witnesses remain. Controlled native
`system_to_matrix_with`/`row_reduce_with` supply exact ranks, parameter exclusions, original-row
witnesses and recovery. Regular parameter values consume retained reductions; exceptional
values re-extract the original equations. Original residuals, domains and bounds independently
check applicability. The source-aware `MathematicalDiagnosticOperation` reaches the ordinary
mathematical publisher through `diagnose_selected_mathematics`.

**Implemented:** true-domain DoubleFloat/Float/ErrorPropagatingFloat diagnostics, certified
nearest rational coefficient/conic conversion, and restricted directed RealBall execution.
Exact source literals, fixed inputs and representation scales retain their canonical binary64
bits as rationals under distinct preparation identities and bounded construction. Compiler
source admission precedes simplification; the supplier independently checks final instructions.
The former selected general-evaluation/`to_f64` conversion branches are replaced. LDLᵀ/Gram,
negative-direction and cubic-root certificates retain their existing authority. Native
`sub_mul_assign` consolidates the Schur update without a measured speed claim.

**Tested, default Nextest, zero failure baseline:** the selected diagnostic/coefficient/class
batch passed 36 tests, and the diagnostic/Gram/root/conversion batch passed 16. The actual
authored accounting and ball value/first/second-partial runtime journey passed. After fixing
shared non-solver numerical-policy publication,
`scripts/pse test pse-runtime --lib -E 'test(mathematical_diagnostic_original_reader)'`
passed 1/1 (`01a1c641-e50a-7667-8a33-2c8f33cd3753`). The independent protected reader preserves
the original precision, formula, source revision and manifest after source evolution and
scientific-service retirement; incomplete and mismatched selections refuse without changing
the stored attempt. These results do not establish native RPC acceptance.

**Cancelled:** the actual HTTP/2 diagnostic journey
`scripts/pse test pse-runtime:native_boundary_acceptance --features engineering-service -E 'test(public_native_diagnostic_rpc_)'`
(`01a1c64e-782c-7160-bbba-f16df973793f`) stopped before compilation, with complete settlement
and no pending native lifetime. It remains required on resumption. D-owned reported Clippy
findings were corrected; the final workspace Clippy verdict remains unestablished, as recorded
in the coordinator handoff.

**Settled bounded decisions:** ordinary elimination suffices for the actual small accounting
block; reopen on consequential coefficient growth or a domain failure requiring a complete
fraction-free/modular/witness route. Interior rational search remains unselected without strict
interior/contraction guarantees. Complex balls require a typed complex consumer; `nsolve` a named
nonlinear estimate consumer; rational explanations an explicit changed-value/difference or
containment demand. These are decisions rather than unfinished substitutes for selected scope.
All resulting claims remain diagnostic: no solver seed permission, physical feasibility or
empirical applicability is inferred. Ball support is limited to the admitted real basic-arithmetic
profile. H retains assembled qualification.

