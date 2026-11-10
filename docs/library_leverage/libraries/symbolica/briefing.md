# Symbolica ecosystem: discovery briefing

Inspected 2026-11-09 at `d9e82627feec6cea5470dbcf1974fd4e8dfe19df`, including
user-supplied Numerica/Graphica overviews and unrelated concurrent Plan72 changes.
**Interface-checked:** representative repository and exact supplier source reads.
**Proposed:** research priorities and composed workflows. No scientific execution or measured
speed, compilation or memory benefit is claimed. [Coverage and evidence](coverage.md) distinguish
fresh defining-package API exports, retained historical reference evidence and fresh census failures.

The useful target is to preserve sharing and mathematical structure through a complete operation,
then select the strongest useful library representation. Current code already delegates expression
normalization, substitutions, joint evaluator construction, aliases, direct translation, dynamic
hyperdual arithmetic and exact roots. This refresh therefore broadens the November2 inquiry from
individual evaluator APIs to transformations, exact algorithms, numerical domains, graph structure
and their compositions. Earlier applicability accounts retain their own dates and limits.

## Topic accounts

| Topic | Research account |
| --- | --- |
| Shared definitions, native traversal, rewriting and function hooks | [Symbolic transformations](symbolic-transformations.md) |
| Polynomial domains, elimination, solution coverage, roots and analytical tooling | [Exact algebra and solving](exact-algebra-and-solving.md) |
| Joint preparation, reusable modules, derivatives and execution backends | [Evaluator composition and execution](evaluator-composition-and-execution.md), [technical contract](evaluator-contract.md) |
| Clean/incremental Rust builds, mathematical preparation, streaming and persistence | [Build, memory and transfer](build-memory-and-transfer.md) |
| Numerical domains, exact matrices, rounding and enclosure certification | [Numerica briefing](../numerica/briefing.md), [domains and certification](../numerica/domains-and-certification.md) |
| Dynamic/fixed derivative shapes, adaptive integration and combinatorial work | [Differentiation and integration](../numerica/differentiation-and-integration.md) |
| Topology equivalence, symmetry, generation and semantic mapping | [Graphica briefing](../graphica/briefing.md), [technical boundaries](../graphica/canonical-generation.md) |
| Tensors, contraction networks, integration companions and mathematical presentation | [Structured ecosystem](ecosystem.md) |

## Priorities and composed workflows

1. **Preserve shared products through transformation and differentiation.** Trace a real pair of
   consumers of equivalent local definitions/partials through the existing prepared owner. Native
   aliases, visitors, collection and reusable transformations could remove repeated construction
   before CSE runs. Different guards, coordinate layouts or derivative demands defeat naive reuse.
   Determine which construction actually repeats after existing caching and sharing.
2. **Use stronger exact domains for a selected mathematical question.** Combine parameter-aware
   extraction, polynomial/rational or matrix algorithms, elimination, original-coordinate checks
   and shared numerical preparation. This could replace repeated coefficient inspection or enable
   useful exact reduction. Exceptional parameter values, retained domain restrictions and
   coefficient growth are central countercases. Identify one small useful block before proposing
   a general preprocessing framework.
3. **Compose independently reusable numerical products.** Native merge and newer composer/import
   interfaces may avoid rebuilding symbolic expressions while retaining module reuse. Compare a
   real pure chain against one joint evaluator, including input maps, lifetime and preparation.
   A single joint build may already be simpler. Non-inlining can conflict with dualization;
   coordinate agreement and real specialization have lifecycle requirements.
4. **Reuse derivative shape and choose arithmetic for the claim.** Dynamic hyperduals already
   share immutable shape; investigate repeated shape construction that survives product reuse.
   Fixed shapes trade runtime preparation for Rust specialization. Precision, error estimates and
   certified balls offer distinct products; certified arithmetic does not establish certified
   transcendental evaluation. Start with a named accuracy-sensitive consumer and independent check.
5. **Enable structure-aware exploration and repeated execution.** Graphica canonical templates or
   constrained generation could feed physical admission and shared/batched evaluators. Parameterized
   mathematical template reuse differs from deduplicating scientific trials. Preserve axis/port
   permutations and actual varying values. Adaptive integration similarly offers a new expectation
   workflow only after its probability measure, weights and failure semantics are defined.

These priorities order attention, not eligibility. Streaming, modular reconstruction, asymptotic
series, symbolic integration, tensors, special functions and mathematical presentation remain
useful directions where their complete workflow has value. Missing current callers do not exclude
new capabilities. Breaking boundaries are eligible when they improve the target and retain required
meaning; a current wrapper or IR is not itself a preservation requirement.

## Handoff

The [current Plan66 owner](../../../plans/66-integrated-simulation-target.md#symbolica-improvement-dispositions)
retains scheduling and acceptance. This discovery creates neither new CBO IDs nor adoption decisions.
For a selected responsibility, deepen its codebase opportunity and applicability, then compare
complete alternatives through the existing leverage routes. Reusable technical accounts remain
separate from contextual hypotheses. No licence was used to exclude a direction.

## Follow-up from the companion addenda

The later [Numerica addendum assessment](../numerica/addendum-followup.md) identifies stable
operations through the whole execution pipeline and precise SIMD eligibility as the strongest
new depth. A bounded exact-arithmetic update inquiry is also credible; directed rational
conversion and low-level polynomial kernels already have substantive supplier delegation.
The [Graphica addendum assessment](../graphica/addendum-followup.md) sharpens mapped template
reuse and bounded closed/open topology generation, including the missing edge-correspondence
problem. It also corrects the earlier directed self-loop factor statement. These are research
priorities, not new adoption or Plan66 execution decisions.
