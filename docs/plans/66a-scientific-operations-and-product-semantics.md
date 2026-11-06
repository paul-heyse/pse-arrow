---
title: "Plan66a: Scientific operations and product semantics"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [design, numerical-execution, validation]
---

# Scientific operations and product semantics

## Remaining target and foundations

Plan66 is the sole go-forward plan. This companion owns the remaining shared scientific
meaning consumed by its delivery companions. It actions review I01/I02/I09 and S01/S07;
the [coordinator](66-integrated-simulation-target.md) owns combined scope and dispositions.
Core3.4, heuristics1.0, process-simulator1.5 and test-assessment1.2 apply.

**Interface-checked:** main `66341514d` already has native
[`DeclaredExecution`](../../crates/pse-runtime/src/workflow/modeling/declared.rs),
[`PreparedOperation`](../../crates/pse-runtime/src/workflow/operation_preparation.rs),
[`Completion`](../../crates/pse-runtime/src/workflow/completion.rs), dimensional admission,
authored scientific applicability and exact task/step/point observation. These are prerequisites,
not new implementation packages. `ProductScope::governing` deliberately refuses a point with
a member binding; it does not implement permission for every experiment/member. Closing that
consumer gap and extending actual scientific contracts are the changes here.

## Design and consuming behavior

Compose an operation from its physical definition/context, selected case, purpose, actual input
roles, product demand and required evidence. Keep steady simulation, fitting, optimization,
control and structural analysis as distinct native operations reusing physical definitions.
Do not make a universal task enum, Proto interpreter or separate policy catalogue. Native
admission decides supported formulation, contextual membership and effects; generated views
describe inputs and observations without reconstructing sealed permission.

Retain the source/context/structure checks and admitted value-only overlays already provided
by `DeclaredExecution::prepared_model`. Make original admission versus editable request
unambiguous where a consumer currently reconstructs that distinction. A value update on an
admitted path can reuse compatible preparation; a changed physical datum, source, layout,
phase policy or derivative premise invalidates the dependent products. Mathematical scratch,
historical interpretation and current provider readiness have different lifetimes.

Complete governing scope for the actual fitting, Study, uncertainty and trajectory consumers:
original run/attempt, experiment/member, step and retained reference point must identify the
producer's product, not a row ordinal or caller label. Preserve independent qualified siblings
when an aggregate fails. Missing, duplicate, foreign or ambiguous governing facts withhold use
while remaining readable evidence. Structural Analyze can publish valid subject/operation
evidence with zero numerical members; do not invent a physical-state requirement.

Extend coordinate and material semantics at their existing authored/modeling owner when a real
consumer needs them: species/phase/site membership, true/apparent composition, datum, physical
time and mesh coordinates. A rectangular numeric array is a realization, not the scientific
coordinate authority. Distinguish physical initial conditions from numerical guesses, scheduled
controls from event/reset semantics, and physical state carryover from Study succession.

Fitting retains shared parameters, experiment-local nuisance parameters and initial states,
observation units/basis/time alignment and noise/dependence. Existing initial-interval
`IntegratedExperiment::Binding` does not authorize a later-interval target by numeric-path
substitution. Extend supported schedule targets only with native admission, initialization,
derivatives and independent correspondence. New physical formulations belong to G/H/I;
this document supplies only contracts genuinely shared by those consuming models.

## Library realization

Use existing source-derived `pse-contract` projection declarations for mechanical exposure;
Buffa0.9.2 descriptors and generated local validation describe shape and constraints, while
native checked constructors retain contextual scientific decisions. See
[CR04 and exact sources](../plan_improvement_reviews/plan64-65-contracts-and-runtime-analysis-2026-11-06.md#CR04).
Do not introduce a second dynamic validator or scientific schema universe.

Salsa0.28.5 tracked preparation preserves consumed semantic dependencies; its creator-owned
`specify` restrictions do not permit an arbitrary mutable cross-owner cache. Numerical values,
structural specialization and mutable sessions must remain separable. See
[S03](../plan_improvement_reviews/plan64-65-scientific-extensibility-analysis-2026-11-06.md#S03)
and [maintained source](../../vendor/salsa/src/lib.rs). B owns generated representation,
C preparation custody, D mathematical realizations and E historical read contexts.

## First consuming agreements — November6

A01's first result delivery consumes existing native selected-result context and governing
permission without inventing a new admission object. The current task/step/point contract
is sufficient for that passive fidelity cutover; experiment/member permission remains A02.
For H03, normalized spatial coordinate and actual cell stock are distinct: a backward N-cell
aqueous cascade has interior cell Amount `V/N*C`, no inlet stock and physical incoming-minus-
outgoing ComponentFlow. A derivative in normalized x includes the mesh scale; combining it
with cell stock requires the corresponding cell-width weight. Independently sum actual cell
stocks and cumulative boundary flow. The scientific owner will first exercise existing indexed
stock/temporal composition and report any exposed shared seam before extending the compiler.
Generic unsupported dynamic/Radau/reactive/energy cases retain their current refusals.

A02's first fitting delivery is integrated. `PreparedFit::assess_candidate` now records each
declared original `InstanceId` into one
[`FitAssessment`](../../crates/pse-runtime/src/workflow/fitting/assessment.rs), whose private disjoint
ranges borrow the sole owned checks/reports. Completed, failed and unattempted members remain
distinct; a later refusal preserves earlier evidence and actually returned partial rows. The shared
typed strategy policy traverses original cause trees so numerical work with terminal cleanup cannot
start another member, recovery rung or profile subdivision. It retains the original envelope and
all causes. The steady observation producer can still lose its own unreturned partial buffer;
that narrower producer gap remains A02 work.

Joined `FitTrajectory` borrows the original run/report/member/assessment. FlatV3 profile documents
borrow retained chains through Serde serialization, avoiding deep copies of physical evidence.
The source-owned registry emits `fit_experiment_assessments` and profile-points interpretation4's
optional original assessment, including explicit historical unavailability. See
[native-math declarations](../../crates/pse-schema/src/catalog/native_math.rs),
[borrowed document](../../crates/pse-runtime/src/workflow/fitting/documents.rs) and proposed
[ADR-0253](../adr/0253-retain-original-fit-experiment-assessments.md).
Missing-column refusal and absent historical metadata have controls; authentic persisted-v3
adoption remains unqualified. These observations confer no new scientific-use permission.

The next native permission delivery must bind the actual original member, report/source and
enclosing run/attempt. A whole-member scope can qualify an original trajectory separately from fit
optimality; a sampled point needs its own retained native coordinate authority. Original physical
time, authored target/source and mesh-site correspondence describe points; row ordinals, callback
caches, caller maps and newly hashed time labels cannot supply authority. Missing demanded accuracy
evidence withholds use. Preserve qualified siblings independently of aggregate fit failure.

The first L02 consumer initially supplied an integration fixture to `CaseOperation`, whose actual
contract is an authored solve with SolveSettings; that request correctly refused. Existing typed
`SimulationOperation` executes the original fixture and retains its physical controls. Preserve
these distinct operation owners rather than infer a need for universal dispatch from that caller
mistake. A01 assesses useful discovery/selection improvements against real consumers. The readable
`simulation_samples` still retain Unknown usability until the actual trajectory-product owner
supplies its governing grant; native numerical acceptance alone does not issue that permission.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| A01 | in-progress | Existing declared/prepared/completion owners; first scientific and developer consumers | First passive result and bounded physical inventory consuming agreements are recorded; remaining operation/input/product gaps follow their actual consumers without replacing working admission. |
| A02 | in-progress | A01; actual fit/Study/trajectory product producers | Original experiment assessment ownership, retained/exported observations and joined trajectories are integrated, including terminal-cause continuation and borrowed-profile repairs. Implement missing member/trajectory governing scope through producers, native permission, storage and independent readers; close the remaining steady partial-buffer producer and authentic historical adoption gaps. No new member permission is yet implemented. |
| A03 | planned | Selected G/H/I physical consumers | Extend material/coordinate and physical operating-program contracts only at exposed shared gaps; retain explicit unsupported combinations and datum/derivative premises. |
| A04 | planned | A01/A03; J mixed experiments | Integrate shared/local fitting roles and selected schedule targets; distinguish admitted value reuse from structural/scientific invalidation in native and portable consumers. |
| A05 | planned | A02/A04; E/F and selected G–K deliveries | Finish changed consumer population and remove superseded mode/scope interpretations; update enduring contracts through applicable decision routes. |

Packages consume narrow working prerequisites rather than entire companions. In particular,
ordinary steady analysis does not wait for distributed or specialized physics.

## Verification and outcome

**Proposed verification:** exercise two same-quantity products at different points/members,
foreign attempts, duplicate steps, failed siblings and zero-member Analyze. Change a value
on an admitted path, then a source/datum/mesh/schedule premise, and discriminate valid reuse
from invalidation. Compare a shared model used for steady simulation and transient inference
with independently specified observation mapping. Add one scientific family without changing
unrelated scalar providers. Family physics and full population acceptance belong to G–L.

**Tested, targeted in the owning checkout:** all-target `pse-runtime` compile with
`solver-diffsol,solver-ipopt,solver-highs,native-study-tests,engineering-service` passed with zero
errors/warnings. The same-feature assessment/profile/strategy selection passed18/18 with complete
capture/settlement, including nested terminal causes, positive numerical continuation, actual LM
chain/point/assessment pointer identity and FlatV3 serialization. Full generation passed15 targets.
Independent follow-up found no residual material defects in those two repairs. Main integration
includes the combined generated world; matching main receiving is separate.

**Outcome:** implementation is active with the bounded assessment delivery above. Shared operation
selection, native member/trajectory use, remaining physical contracts and full consumer population
remain open. No full A or Plan66 qualification is claimed.
