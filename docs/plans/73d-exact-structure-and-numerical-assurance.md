---
title: "Plan73d: Exact structure and numerical assurance"
status: draft
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

The inspected planning baseline is `baadc7e32`. The source review records its earlier
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
`shape_facts_with_objectives`; extend that owner rather than introduce a second cache. Actual
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
| D01 | open | Coordinator decision route; A's admitted original-product and binding contract | Exact/assurance request, claim and refusal semantics implemented with source/domain attribution. |
| D02 | open | D01; A's working selected-product demand/lifetime | Class evidence and coefficient projection consume compatible shared exact views; replaced repeated extraction deleted; sparse demand preserved. |
| D03 | open | D01–D02; native matrix condition/provenance support | Selected parameter-block diagnostic delivered at ordinary/exceptional values; reduction investigation disposition and any selected implementation complete. |
| D04 | open | D01; A's working exact evaluator product; B's supplier-qualified mapping/error contract | Chosen-precision and propagated-error diagnostics integrated without stronger false claims; certified conversion consumers migrated. |
| D05 | open | D01; D04's claim/result path; checked supplier bridge and provenance | Real supported arithmetic enclosure executable through native evaluator; unsupported program and singular interval outcomes explicit. |
| D06 | open | D02–D05; E's protected presentation consumer when exposing mathematical output | Witness/root/conversion preservation, conditional dispositions, original-result readers and all in-scope replacement deletions accepted. |

### D01 — Admit exact and assurance requests

Implement selection through the existing compiler/model scientific operation owner, consuming
resolved physical and mathematical source products. Extend registry declarations for actual
request/result relations and regenerate through `just codegen` during execution; generated
files are never hand-edited. Keep requested scope, examined scope, parameter roles, precision,
input uncertainty and actual claim separate. A partial/unsupported selection retains its source
and reason; the independent native host remains command/process custody, not scientific policy.

Migrate exact diagnostic operation entry points and their result producers together. Test a
valid selected law, a provider or branch lacking the requested product, duplicate or absent
unknowns, a source-domain violation hidden by algebraic cancellation and an interrupted request.
A changed source cannot make a retained old result appear current. The result contract is also
the small early prerequisite consumed by E; E need not wait for every D realization.

### D02 — Share exact extraction at actual consumers

Refactor the existing request aggregation/objective owner into selected reusable exact views
without changing original coordinates or domain authority. Move `class_evidence` and
`coefficients` onto that owner, including repeated objectives/constraints and compatible exact
inspection. Preserve consumed fixed-value assumptions and sparse allocation/refusal checks.
Delete superseded duplicate conversion, aggregation or differentiation paths and their
mechanism-specific fixtures as soon as the replacement's targeted tests pass. Preserve useful
behavior tests against the target; do not retain old code as a comparison backend.

Acceptance must include two real consumers of the same compatible view, a mismatched
parameter/guard refusal and a wide sparse one-row demand. Verify independent exact expected
coefficients and affine/quadratic classifications, including cancellation between contributions.
Do not use a new snapshot produced by the same extractor as the only expected answer.

### D03 — Deliver parameter diagnostics and settle reduction

Integrate the matrix composition above into a selected modeling diagnostic, with named original
rows and parameters carried through the result. Implement generic condition projection in the
supplier if needed, or explicitly limit unsupported generic scope while still delivering exact
specialization results. Settle small-block reduction against the accounting example and its
original-coordinate validation. An affirmative reduction decision includes its implementation,
coordinate recovery, migrated consuming path and deleted replaced elimination; a negative
decision names the failed criterion and reopen trigger in this plan.

Acceptance distinguishes the three `p,q` cases, generic exclusions, invalid original denominator,
nonlinear unsupported extraction, physically inadmissible algebraic solution, selected-row
limits and cancellation. Verify classifications independently from small exact matrices. This
operation does not replace structural rank analysis, solver selection or whole-model presolve.

### D04 — Deliver approximate diagnostics and one certified conversion boundary

Map the selected evaluator once for each requested arithmetic realization, preserving exact
constants until that mapping. Bind selected inputs without rebuilding admitted source
preparation. Keep mapped workspaces request-owned and release them with the result operation;
retain compact outputs and premises, not all supplier stacks. Add the supplier fallible mapping
needed to turn unsupported constants/callbacks into typed refusals. Migrate actual diagnostic
and proof-projection clients to the shared conversion owner; delete their alternate rounding
policies where replaced. Ordinary production binary64 solver semantics remain explicit.

Use a cancellation-sensitive arithmetic relation, an exact rational rounding tie, signed zero,
subnormal and overflow boundaries, and a provider with only binary64 support. Independently
expected exact/multiprecision values must reveal both wrong arithmetic and overstated claims.
An error estimate or extra digits must never populate an enclosure claim field.

### D05 — Implement and qualify the restricted enclosure bridge

Add the maintained supplier `EvaluationDomain` integration and checked arithmetic-program
constructor described above, with no implicit callback/constant fallback. Reuse native evaluator
instructions and ball arithmetic; enforce restrictions at construction so a later generic map
cannot accidentally bypass eligibility. Expose the checked product through the diagnostic owner,
not raw unchecked ball evaluators as a scientific certificate API. Include maintained-source
notes and focused supplier tests in the change; coordinate shared vendor edits with B.

Test exact rational polynomial containment, input intervals versus rounding uncertainty, a
nonzero rational denominator and a zero-crossing denominator. Include an expression where a
forbidden operation disappears under normalization, so source-domain/support admission and
final-program validation are both exercised. A `sin`, `log` or external callback request must
refuse enclosure even if approximate evaluation works. Check unsupported instructions directly
at the supplier constructor as well as through the product operation. No application interval
walker or transcendental FBBT replacement remains after cutover.

### D06 — Complete consumed certificates, readers and dispositions

Verify that exact class/coefficient clients retain the LDLᵀ/Gram and negative-direction evidence
they consume, and that maintained bounded roots and binary64 conversion keep their established
claim strength. Consolidate thin arithmetic only where useful; delete a superseded arithmetic
helper when the target tests prove its replacement. Carry exact and assurance fields into the
existing protected original-result inspection/export path, using E's presentation projection
without recomputing the mathematics while reading. Match source, original attempt and actual
claim; bounded readback may be incomplete without weakening the stored outcome.

Record decisions for selected block reduction, fraction-free/modular alternatives, complex-ball
extension, rational interior-search, approximate `nsolve` and rational explanation candidates.
Each conditional lead closes as adopted-and-delivered or explicitly not selected with its
criterion, evidence and observable reopen condition. It is not an unbounded research backlog or
a reason to defer the selected exact diagnostic/enclosure operation. Any new evidence/kernel
contract follows the coordinator's decision route before implementation relies on it.

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
