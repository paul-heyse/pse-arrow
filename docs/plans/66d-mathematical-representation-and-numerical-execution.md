---
title: "Plan66d: Mathematical representation and numerical execution"
status: planned
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [design, numerical-execution, performance, validation]
---

# Mathematical representation and numerical execution

## Remaining target and existing strengths

This companion actions review I03/I06 and S02–S04/S07 under the
[Plan66 coordinator](66-integrated-simulation-target.md). Blueprint §7/§14–§18 and
PS-05–PS-10 supply mathematical context. Main `66341514d` already contains sparse
symbolic/numeric separation, implicit response actions, shared multi-output evaluators,
selective accuracy arithmetic, SVD fitting covariance and class-specific native runners.
Do not recreate those capabilities. Remaining work selects stronger representations at
actual demanded blocks, completes changed consumers and verifies the current maintained
arithmetic closure beyond its focused tests.

## Selected design and critical library functionality

Preserve original coordinates, exact constants, source correspondence, domain guards and
scientific acceptance through symbolic simplification and numerical execution. Start from
the requested product and mathematical class; choose symbolic/coefficient/factorable/sparse
or action-based realization before introducing caches or pools. A bounded/constrained solve
cannot become plain Newton iteration merely because the latter is easy to call.

Maintained Symbolica3.0.1
[`export_instructions`](../../vendor/symbolica/src/evaluate/instruction.rs) permits
library-owned evaluation programs to cross an admitted mathematical boundary;
[`map_coeff_with_prec`](../../vendor/symbolica/src/evaluate/evaluator.rs) changes coefficient
representation with explicit precision. Use these with existing multi-output preparation
to avoid repeated expression conversion and to retain high-precision constants where demanded.
Compare selective affine/rational elimination against retained numerical blocks on actual
consumers. Carry pivot/nonzero assumptions and recovered original variables; do not expand
exact algebra globally or discard guards. The
[Symbolica overview](../libraries/scientific_calcs/symbolica.md) supplies broader leads;
maintained source defines the consumed contract.

Keep precision/error claims explicit: repeated high precision, backward error, conditioning,
validated enclosure and independent physical agreement answer different questions. Basic ball
arithmetic is not a general transcendental enclosure, and series-to-expression conversion
does not retain a remainder automatically. Review pending accuracy opportunities by demanded
claim; do not prescribe one arithmetic mechanism to every solver. Physical budgets or comparison
allowances may change with dimensional/scientific rationale and revealing countercontrols.

For a selected large response, investigate maintained faer0.24.4
[`LinOp`/`BiLinOp`](../../vendor/faer/src/operator/mod.rs) and
[`lsmr`/`lsmr_scratch`](../../vendor/faer/src/operator/lsmr.rs) under the existing native
response-action owner. These provide multi-RHS actions and explicit scratch/parallelism.
Select an action-based route only after comparing the actual consumer with retained factors:
repeated RHS often favor factors; a large sparse problem may favor preconditioned actions.
The [maintained arithmetic contract](../../vendor/faer-arithmetic.md) qualifies an exercised
binary64 closure, not every generic scalar or iterative method. Preserve the current
[direct weighted-response SVD](../../crates/pse-runtime/src/workflow/fitting/covariance.rs)
and its rank/withholding rules; do not form normal equations to simplify a large-response path.

At ndarray0.16.1 use `ArrayView`, `ArrayViewMut`, `Zip::and_broadcast`/`for_each`; at faer use
`MatRef`/`MatMut` and explicit scratch. These admit borrowed/strided data and fused traversal.
See [S04](../plan_improvement_reviews/plan64-65-scientific-extensibility-analysis-2026-11-06.md#S04)
and [ndarray reference](../libraries/scientific_calcs/ndarray.md). Preserve species/phase,
units/datum and ordered coordinate correspondence at the semantic adapter. Compare packing
with strided views on a real batch; zero-copy alone is not a throughput result. Charge retained
coefficients, factors and scratch to their actual native owner.

Native solver selection remains by full problem class and supported derivatives/domain:
KINSOL square systems, Ipopt/POUNCE smooth NLP, HiGHS linear/mixed-linear, Clarabel admitted
cones, SCIP factorable mixed nonlinear, and Diffsol/IDAS admitted dynamics. Use the selected
[native solver reference](../../.codex/skills/native-solver-libraries/SKILL.md) and pinned
source contracts for the route actually changed. No new fallback chain or solver family is
selected by this plan. Higher-index/hybrid derivative extensions need a concrete H/I/J consumer,
exact consistent-initialization/event contract and its applicable decision before activation.

### Canonical reconstruction activation

C04's canonical binary Recipe receiving requires a justified pre-allocation envelope for
Symbolica normalization, coefficient and derivative/partial growth, and global symbol
registration. A bounded decoder, post-construction node/slot limit, payload multiplier or
owned child process does not establish that bound. D02 supplies the mathematical construction
contract and C04 the receiving admission/receipt/custody contract. Retain current JSON v3
receiving until bounded reconstruction or a qualified alternative satisfies those obligations,
or a specifically approved revised contract replaces them. The already implemented provider-free
compiled f64 Value/First live-child handoff is distinct and cannot close canonical reconstruction.
This is an operation admission contract, not a new host-wide memory cap.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| D01 | planned | Actual remaining G/H/I/J blocks; current source-owned evaluator/accuracy contracts | Select representations and precision by demanded products; decide selective elimination/coefficient mapping versus retained block with original-coordinate recovery and guarded validity. |
| D02 | planned | D01 selected transforms; A semantic dependencies and C preparation | Integrate chosen evaluator/batch/layout changes through preparation, native demand, product/error reporting and transfer where consumed; resolve canonical reconstruction's pre-allocation growth contract before C04 activation; delete replaced staging and duplicate evaluators. |
| D03 | planned | A selected large-response consumer; current direct factor/SVD route | Decide factors versus faer actions using correctness, convergence/preconditioner, scratch and whole-operation evidence. Deliver the action route only if justified; otherwise retain factors with the comparison and revisit trigger. |
| D04 | planned | Changed maintained arithmetic/provider consumers | Complete receiving controls for actual SCIP/Diffsol/KINSOL/NLP/native-root closures affected by changes; add missing invocation-bound regime/selection observation under ADR-0251's selected contract. Source/preparation logs cannot stand in for winning observations. |
| D05 | planned | D02–D04 selected deliveries; family consumers | Retire replaced numeric paths after targeted controls and caller migration; supply complete-operation comparisons and enduring mathematical contracts to L. |

D03 is a decision obligation, not a mandatory second backend. Likewise expression streaming,
new tensor machinery or another property/DAE library requires a demonstrated remaining consumer;
record the concrete gap before adopting it. Existing native mathematical meaning stays authoritative.

## Verification and outcome

**Proposed verification:** changed coefficients with unchanged structure; changed sparsity,
phase regime and precision; ill-conditioned/rank-deficient cases; incompatible strided/ragged
layouts; rejected original domain; multiple RHS; cancellation and escaped scratch/outputs.
Compare original-coordinate residual/actions with a retained direct route and independent
physical/reference expectations. Structural equivalence and a library convergence flag do not
prove physical validity. Run targeted controls as implementation lands, and receiving native/
scientific campaigns through L at functional scope end. Performance benefits remain unmeasured.

**Outcome:** planned; existing passing controls retain their original conditions, not new
whole-plan qualification. No product change or new numerical measurement occurred in authoring.
