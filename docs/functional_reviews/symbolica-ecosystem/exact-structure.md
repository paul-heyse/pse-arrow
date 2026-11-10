# Exact structure, reduction and scientific certificates

Supporting assessment for [the integrated review](../symbolica-ecosystem-functional-improvements-2026-11-09.md), inspected at `7d89d16f0` on 2026-11-09. **Interface-checked:** maintained exact conversion and current mathematical consumers; **Proposed:** recommendations below. No numerical experiment or performance measurement. Core3.4, efficiency heuristics1.0 and process-simulator1.5 shape the judgment (AP-03/04/07, DP-08/10/13, PS-04/06/09/10).

## O04 — Request-shaped exact products and explainable preprocessing

**Recommend** sharing exact products within the admitted mathematical request and exposing a bounded exact-block diagnostic; **conditionally recommend** reduction of a selected small affine block. The useful outcome is that a developer or modeler can ask for coefficient structure, parameter degeneracy or accounting consistency without implementing another coefficient extractor or treating numerical failure as an explanation.

Current `pse-math::class_evidence::prove_affine` uses symbol discovery, derivatives and zero substitution to obtain sparse global-coordinate rows. `shape_facts_with_objectives` and `coefficients.rs` already reuse exact request-local objectives and build exact degree-at-most-two polynomial products. This is not absent delegation. The remaining opportunity is a consumer-shaped exact view shared by compatible evidence/diagnostic/export uses, without repeatedly expanding or converting the same qualified expressions.

Symbolica `system_to_matrix` supplies a genuinely stronger option for parameter-dependent linear blocks. Its coefficient field is `RationalPolynomialField<Z,E>`, not scalar Q: parameters remain symbolic. Numerica field solving, rank and consistency operations can consume that representation. A proposed diagnostic could distinguish an ordinary full-rank block from a parameter specialization that makes two conservation equations redundant or inconsistent. It must report what was established, including generic coverage and excluded parameter/domain conditions, rather than silently classifying every specialization.

Compare three complete options:

| Option | Value | Cost or decisive countercase |
| --- | --- | --- |
| Keep sparse derivative-based extraction | Best fit for selected numerical evidence rows and already admitted demand. | Does not by itself provide a reusable parameter-dependent algebraic diagnostic. |
| Shared request-owned polynomial/matrix product | Removes repeated extraction where multiple consumers actually coincide; lets exact domain algorithms perform the complete operation. | A universal matrix over every variable could do more work and retain more backing than sparse evidence. |
| Automatic broad symbolic presolve | Could eliminate numerical unknowns. | Branch conditions, coefficient swell, physical feasibility and inverse coordinate recovery can dominate; not recommended as a default. |

The preferred target retains one qualified exact view only where its consumers share definitions, coordinate support and lifetime. Add parameter-aware extraction when that richer product is needed; do not replace sparse evidence merely to standardize all extraction. For selected elimination, recover original coordinates and evaluate original residual/domain obligations before relying on a numerical result. Fraction-free integer/polynomial elimination requires deliberate denominator clearing and original-domain checks; integer `2x=1` is not a successful integer solve. Modular evaluation, CRT and rational reconstruction are options for serious coefficient swell, not a required second execution engine.

**Interface and rule impacts.** A request for exact structure must declare unknowns, parameters, selected rows and the intended evidence/diagnostic product. Original physical admissions and source occurrences remain meaningful. This can change compiler mathematical-product and diagnostic contracts; material class/fidelity changes require the existing decision route. It does not authorize treating algebraic rank as structural matching or replacing class-specific native iteration.

**Delivered behavior.** A small parameterized block produces useful rank/consistency information at ordinary and exceptional values; selected reduction maps a verified solution back to the original equations. An independent exact example must distinguish inconsistency from underdetermination and generic coverage. Compare a sparse one-row demand where keeping current extraction is preferable. A real second consumer must demonstrate claimed shared-product deletion; no numerical speed claim follows from API availability.

## Arithmetic consolidation without replacing the certificate

For exact convexity, **retain symmetric-pivot LDLᵀ/Gram semantics** and make thin arithmetic consolidation an optional part of O04. Current `convexity.rs::ldlt` owns pivot choice, exact classifications, work/cancellation and retained witness construction. Numerica `RingOps::sub_mul_assign` can express its product/subtract step. The rational implementation specializes denominator-one operands; general rational arithmetic still forms a product then subtracts. This may simplify local arithmetic code, but cannot justify a claimed general fused speedup.

Do not replace this certificate with determinant/rank tests or an arbitrary matrix solver. None supplies the consumed symmetric Gram/negative-direction witness. A fraction-free certificate algorithm would be a different complete computation; its coefficient-growth and witness benefits would have to justify replacing the current algorithm, not follow automatically from fraction-free solve support.

High-level Symbolica already invokes Numerica's multiplication, geometric-sequence and exact-division kernels. **Retain that delegation**, and improve surrounding representation/conversion only for a concrete repeated consumer or missed useful dispatch. A direct application kernel wrapper would otherwise duplicate supplier machinery. Owned integer division and bulk multi-product accumulation remain eligible where an operation genuinely stays integral.

## O05 — Assurance that states the claim, not merely the number type

**Recommend** retaining one exact-to-binary64 conversion owner and building richer diagnostic claims on actual supported domains, not broad precision escalation. Current `rational_binary64.rs` already uses directed `Float::from_rational_round` conversions and explicit binary64 rounding checks, including subnormals/overflow. Current cubic work uses maintained bounded square-free/isolation APIs and a local admitted midpoint refiner. Historical fallback-ladder hazards are dated evidence, not a description of the current implementation.

A modeler may need a numerical estimate, an error-sensitive diagnostic, a supported-arithmetic enclosure, a selected algebraic root or a correctly rounded conversion. Those are distinct results. Numerica balls certify supported arithmetic but not every inherited transcendental operation; exact algebraic objects do not by themselves certify every numerical conversion or parameter specialization. Do not replace full transcendental FBBT or root selection with generic ball evaluation.

For exact root refinement, keep the bounded midpoint route as the ordinary choice. `Rational::round_in_interval` can select a simple exact trial point without approximating model coefficients, but its closed-interval behavior can return an endpoint and guarantees no halving. A new bounded interior-search strategy is conditional on a useful exact-search consumer and a complete contraction/limb-bound argument. An endpoint screen plus retry ladder is not a demonstrated simplification. General `nsolve` remains a possible explicitly approximate research diagnostic, not a substitute for selected/scaled solver convergence or certified root evidence.

**Delivery and integration.** A richer claim must specify coordinate/domain support, approximation or enclosure meaning and inconclusive outcomes. Demonstrate the independent mathematical claim at a boundary case, not simply a higher-precision printed answer. O05 shares numerical diagnostic infrastructure with O02/O10 but does not require changing every production evaluator's coefficient domain.

## Sources

[Integrated exact-domain source account](../../library_leverage/libraries/symbolica/exact-domain-operations.md), [Numerica domain contracts](../../library_leverage/libraries/numerica/domains-and-certification.md), and [historical Symbolica root/resource types](../../library_leverage/libraries/symbolica/README.md) supply dated capability evidence. Current source reads: `crates/pse-math/src/{class_evidence.rs,coefficients.rs,convexity.rs,rational_binary64.rs,implicit_cubic.rs}`, maintained `vendor/symbolica/src/solve.rs::system_to_matrix_impl` and relevant Numerica matrix/rational domain contracts. [Plan66's Symbolica owner](../../plans/66-integrated-simulation-target.md#symbolica-improvement-dispositions) retains existing scope and acceptance. No status is transferred into this review.
