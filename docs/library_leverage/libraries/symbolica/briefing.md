# Symbolica: discovery briefing

Inspected 2026-11-02 at source `7a2f73342`, including the concurrent working tree.
Current-use statements are **Interface-checked** representative reads, not a generated census
or a refresh of every older applicability conclusion. Research priorities are **Proposed**.
No scientific execution or performance measurement was run.

## Capability territory and current usage

| Family | Current observation and research relevance |
| --- | --- |
| Expressions, rewriting and differentiation | The symbolic engine provides the expression representation and transformations. Its full pattern/rewrite surface remains eligible beyond familiar parser and derivative routes. |
| Optimized evaluators and code generation | [Math library integration](../../../../crates/pse-math/src/library.rs) exports evaluator instructions once per floating representation and derives storage/eligibility facts. Export is already used; recommending first adoption would be mistaken. |
| Exact polynomial algebra and roots | The [maintained supplier delta](../../../../vendor/symbolica-bounded-roots.md) adds bounded square-free and isolation overloads to the 3.0.1 source. This local surface differs from the upstream API at the same version. |
| Coefficient domains and numerical composition | Exact coefficients, floating mapping, series and derivative representations offer useful compositions. Mathematical equivalence alone does not establish identical floating-point realization. |
| Streaming, matrices, tensors and companion systems | The [broad overview](../../../libraries/scientific_calcs/symbolica.md) supplies territory to investigate. This pass establishes no complete local-use or configuration verdict for those families. |

## Research priorities

1. **Explore shared multi-output evaluator preparation.** `Atom::evaluator_multiple` can optimize
   expressions together and share subexpressions. This could matter where related outputs are
   prepared independently. First establish the existing grouping and reuse boundaries, then ask
   whether preparation, memory lifetime and numerical policy permit combining them. Do not
   infer redundancy from multiple evaluator objects alone. [Evaluator contract](evaluator-contract.md).
2. **Reuse exported representation facts across consumers.** `ExpressionEvaluator::export_instructions`
   exposes a structured program. The current adapter already shares an inspection copy; next
   investigate whether other selected consumers can use the same facts without losing authored
   obligations or fidelity. The optimized stream is not a replacement for every domain IR.
   [Existing instruction type and applicability](README.md#symbolicaevaluator-instruction-interchange).
3. **Review library-owned exact algebra before expanding local scientific mechanisms.** GCD,
   square-free decomposition and root isolation provide substantial algorithms. Distinguish
   supplier capability from operational contracts such as cancellation and pre-allocation
   admission. The maintained bounded overload is material counterevidence to older claims that
   the local integration has only the unbounded upstream API. Further adoption must preserve
   exactness and the existing supplier/consumer division.

Broad rewrite rules, coefficient-domain conversions, series, compiled evaluators and out-of-core
facilities remain research candidates. Label released, local-maintained and upgrade routes
separately; upstream documentation does not describe every local addition or numerical policy.

## Next evidence and handoff

Generate `symbolica` family evidence from the exact path-overridden supplier and selected native
world. Verify source identity before joining it to public upstream documentation. A fresh reader
should explain which exported evaluator transformations can alter expression shape and why the
bounded root overload cannot be inferred from version number alone. Preserve existing opportunity
IDs and their original evidence dates when later refreshing applicability.
