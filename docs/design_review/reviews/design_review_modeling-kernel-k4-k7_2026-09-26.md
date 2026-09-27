# Modeling kernel K4–K7 target review

## Scope and decision

Author review under core 3.0/process-simulator 1.1 and the pse-arrow binding.
**Proposed: Accept at design level** with the corrections below. This is not independent
review or implementation qualification. The [execution packet](../../plans/21-modeling-kernel-k4-k7-execution.md)
owns correction status; Plan 21 owns broader findings.

## Architecture and behavior

AP-01–AP-06 are satisfied at the proposed boundary: pure declarations, separate library
integrations and attempts, composition through existing executable cases, one registry,
explicit capabilities/outcomes and locally testable policies. G1–G9 and PS-G1–PS-G3 are
satisfied at design level through physical admission, source identities, transformation
lineage, structural matching, declared derivatives, bounded attempts and original-space
qualification with independent closure. Runtime evidence is pending.

Inspection covered finite specialization, whole-root projection, CasePlan, provider
contracts, KINSOL constraints/callbacks, initialization, dynamics, numerical policy and
results. Scientific models and native builds were not qualified.

## Adopted corrections

| ID | Cause and consequence | Correction and verification |
|---|---|---|
| <a id="f01"></a>F01 | Finite expansion cannot enumerate an interval | Resolve continuous realization before enumeration; exactness/lineage tests |
| <a id="f02"></a>F02 | KINSOL constraints encode signs rather than boxes | Trial guards and final interval/residual checks; native recovery tests |
| <a id="f03"></a>F03 | Smooth regimes do not establish smooth selection | Certify boundary jets or refuse derivative demand; ties/switch/singularity controls |
| <a id="f04"></a>F04 | Normalization erases cancelling terms | Retain original term boundaries; cancellation controls |
| <a id="f05"></a>F05 | Parameterized definitions need concrete fixtures | Require authored bindings, automatically attach checks; missing-fixture control |
| <a id="f06"></a>F06 | Local solver failure does not certify infeasibility | Typed inconclusive diagnostics and bounded searches |

## Library fit and alternatives

**Interface-checked:** pinned Symbolica 3.0.0 roots/derivatives, faer 0.24.4 sparse solves
and dense SVD, KINSOL, Diffsol/IDAS, HiGHS, pounce-presolve and petgraph contracts were
inspected during planning with their skills and source. Largest-value partial SVD is
not a smallest-value diagnostic. No runtime or performance result is claimed.
Existing CasePlan and Salsa absorb compilation; no second evaluator, general theorem
prover, Newton solver or scientific type hierarchy is needed. Library state remains
behind its integration owner and outside pure queries.

## Authority and evidence

ADRs 0098–0101 remain proposed; the authorized target is implemented in the execution
packet. Positive/negative synthetic controls accompany mechanisms. K8/K9 retain science
and campaign acceptance; the architecture collection is not amended in passing.
