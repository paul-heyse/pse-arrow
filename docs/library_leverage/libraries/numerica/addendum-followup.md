# Numerica addendum: worthwhile follow-up inquiries

Inspected 2026-11-09 at `924b0c3b28e749b9cd94ad06321b9547c6ff992c`, with parallel
Plan72 source/documentation changes present. The [advanced capability addendum](../../../libraries/scientific_calcs/numerica.md#numerica-rust--advanced-capability-addendum)
was read completely, starting at its supplied line1492. This refresh supplements the earlier
[discovery](briefing.md); it does not redate its entire coverage.
**Interface-checked:** selected locked Numerica 3.0.1 and current consumer source.
**Proposed:** priorities, possible benefits and research questions. No compilation, scientific
test, benchmark or new semantic census was run. This is discovery, not adoption selection.

## Highest-value new depth: stable operations and packed evaluation

### Preserve numerical stability through a complete mathematical operation

The addendum's `log1p`, `hypot`, `sech`, `csch` and sign/exceptional-value interfaces expose
more than alternative spellings for elementary formulas. They can avoid losing a small increment
or overflowing an intermediate even when the mathematical result is representable. A follow-up
should examine whether the mathematical execution pipeline can retain this useful structure
through transformation, differentiation and backend selection.

There is a concrete starting case: [guarded math controls](../../../../crates/pse-math/src/guarded_tests.rs)
construct `log(1+x)` in `high_precision_real_values_survive_optimizer_and_jet_profiles`.
The authored primitive vocabulary currently exposes Log rather than Log1p. The existing test's
absolute-error scale does not establish relative accuracy when the answer is tiny. Native
Numerica f64/F64 and multiprecision implementations provide specialized `log1p`; simply applying
one after `1+x` has already rounded cannot recover the lost increment.

The useful review unit is therefore expression recognition or a stable native function contract
→ symbolic derivative products → interpreted realization → compiled export, with original guards
and domains intact. Examine tiny positive and negative increments, approach to the real-domain
boundary, and First/Second demand against an independent mathematical reference. A one-off
custom scalar workaround would not establish a coherent replacement pipeline. New function
vocabulary or supplier changes may be necessary; their availability is not established by the
numerical trait method alone.

### Review SIMD as a complete operation contract

The earlier discovery nominated batching. The added material makes a focused eligibility review
worthwhile before any backend selection. Locked `domains/float/simd.rs` actually implements
`f64x2`, `f64x4` **and `f64x8`**. It still has six unimplemented hyperbolic/inverse-hyperbolic
methods. Moreover, default `Real::log1p` calls `asinh`: a generic stable-log implementation can
therefore reach a panic on the packed type despite the `Real` implementation. Integer power
uses `powf_simd`, with a debug exponent bound, rather than repeated integer multiplication.

A useful consumer is one provider-free property/residual function evaluated over independent
rows. Investigate its entire reachable operation set, including trait defaults, before treating
SIMD as another interchangeable scalar domain. Cover negative bases and integer powers, zeros
and signed zeros, finite/error behavior, packed tails, derivatives and the current FP profile.
`hypot`'s fallback for a type without a scalar real comparison also deserves inspection: the
trait's stable scalar intent does not establish identical protection for packed values.

Compare interpreter-level packed arithmetic with current scalar and supplier-generated SIMD
routes. Include data packing, demanded outputs and preparation lifetime. This could supply
batching without another expression optimizer or runtime native compiler, but that is a
hypothesis until the exact evaluator domain/function coverage is established. Irregular guarded
or fallible-provider work is a countercase to unconditional batching. No speedup is measured here.

## Bounded exact-arithmetic inquiry

[Exact convexity](../../../../crates/pse-math/src/convexity.rs) retains a necessary
symmetric-pivot LDLᵀ/Gram certificate owner. Its Schur update currently forms a rational product
and then subtracts it from an accumulator. `RingOps::sub_mul_assign` offers a native expression
of that whole update. This is a concrete seam for reducing temporary arithmetic machinery
without replacing pivoting, budget/cancellation or the certificate's meaning.

The important countercondition is in locked `domains/rational.rs`: its accumulation specialization
optimizes the denominators-one case; general rational coefficients still require a product and
subtraction. Thus a small integration simplification is plausible, but a general allocation or
speed improvement is not established. Inspect actual pivot denominators and backing reuse before
promoting the opportunity. Integer-domain owned division and `sub_mul_assign_many` are more
interesting where a complete exact operation can remain in the integral domain.

Fraction-free reduction, matrix content/primitive normalization, coefficient mapping and
entrywise differentiation strengthen the existing exact-block hypothesis. They warrant a deeper
comparison when a named affine block, conservation diagnostic or polynomial coefficient system
needs them. Compare content reduction → fraction-free or modular elimination → original-coordinate
verification against existing exact field operations. `solve_fraction_free` solves in its original
domain; integer `2x=1` must not be mistaken for a request for rational `1/2`. Matrix differentiation
is not differentiation through a solve, and arbitrary entry mapping proves no homomorphism.
These algorithms do not replace a symmetric inertia/Gram witness by virtue of computing rank.

## What does not need a separate review now

| Addendum family | Assessment relative to the existing discovery |
| --- | --- |
| Directed rounding and exact-to-floating enclosures | Already used substantively by [rational binary64 conversion](../../../../crates/pse-math/src/rational_binary64.rs), including Down/Up conversion. Reuse may merit review for another consumer with the same claim; this is not missing adoption. |
| Certified balls, precision, exact reconstruction of stored floats | Existing domain/certification account already distinguishes arithmetic certification, unsupported transcendentals and intended value versus stored approximation. New explanatory detail does not remove those constraints. |
| Polynomial multiplication/division and geometric-sequence kernels | Maintained Symbolica already invokes these in polynomial, homogeneous, factorization and GCD operations. Follow the high-level supplier path first. A new custom bulk consumer or missed supplier dispatch would justify deeper work; direct low-level adoption alone could duplicate delegation. |
| Rational size/error approximation | Useful for explicitly approximate reports or scientific candidate discovery. It changes coefficients; selecting a simple rational within a certified enclosure does not establish identity. No such new consumer was established in the inspected math scope. |
| Ring printers, wrappers, borrowed/owned utilities | Useful integration tools for a concrete exact-domain consumer; no standalone high-value replacement was found here. Domain context cannot be dropped for convenience. |
| CSR, Python adapters and adaptive integration | Existing discovery already covers CSR and stochastic integration. Python adapters may help isolated reference tooling, but provide no reason by themselves to change the product boundary. |

All addendum sections1–15 and their compositions were considered. The strongest follow-up is a
single numerical-realization inquiry covering stable operations and SIMD together, plus a bounded
exact-update inquiry. Exact block elimination remains a worthwhile conditional direction rather
than a newly selected implementation target.

## Reproducible source scope

Locked Numerica 3.0.1 source: `domains/float.rs`,
`domains/float/{simd,native,multiprecision}.rs`, `domains/{rational,integer}.rs`,
`tensors/matrix.rs`; [versioned source index](https://docs.rs/crate/numerica/3.0.1/source/src/).
Supplier dispatch inspection covered `vendor/symbolica/src/poly/{polynomial.rs,polynomial/homogeneous.rs,factor.rs,gcd.rs}`.
Consumer source links above identify the representative operations. No spelling-based search is
promoted to repository-wide absence, and original API acquisition limits remain in the
[integrated evidence account](../symbolica/coverage.md).
