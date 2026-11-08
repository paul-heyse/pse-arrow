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
a member binding; that baseline did not implement whole original-member permission. The bounded
fit and standalone deliveries below extend it; remaining point/product and consumer gaps stay open.

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
all causes. The steady observation producer now retains its completed rows in the original
caller-owned buffer. The selected fitting control at `7b194b345` checks the original
typed retirement causes and partial rows through export, preserving unattempted successors.

Joined `FitTrajectory` borrows the original run/report/member/assessment. FlatV3 profile documents
borrow retained chains through Serde serialization, avoiding deep copies of physical evidence.
The prior delivery emitted `fit_experiment_assessments` and profile-points interpretation 4.
The current working tree advances profile points to interpretation 5 and requires the
original assessment and callback terminal flag. See
[native-math declarations](../../crates/pse-schema/src/catalog/native_math.rs),
[borrowed document](../../crates/pse-runtime/src/workflow/fitting/documents.rs) and proposed
[ADR-0253](../adr/0253-retain-original-fit-experiment-assessments.md).
Finish generation and test the current producer and receiver together. The predecessor
profile-reader obligation is removed. FlatV3 remains the current fit document format.

The first native permission delivery is integrated: `ProductScope::Member` and the output-only
[`MemberScientificPermission`](../../crates/pse-runtime/src/workflow/completion/member.rs)
bind an original fit experiment to its actual run/attempt, source/physical/preparation and private
report/assessment custody. `FitTrajectory::qualified_report` consumes that exact correspondence;
raw failed evidence remains readable and decoded snapshots cannot mint live proof. Local native
completion and missing-accuracy refusal remain the decision owners, independently of aggregate
fit optimality or an unrelated sibling failure. Actual converted candidate parameters are retained
by their existing assessment owner and projected through the original source/layout; no input map
is recreated by an exporter. Proposed [ADR-0254](../adr/0254-govern-original-fit-trajectory-members.md)
records this bounded contract and the next standalone/protected consumer. Native `scripts/pse test
pse-runtime --lib --features solver-diffsol,solver-ipopt,solver-highs,native-study-tests,engineering-service`
selected fitting, snapshot and actual durable-attempt controls passed 9/9 in the owning checkout, zero failures/build
warnings, complete capture/settlement (`01a1b349-a547-73c7-b3f3-ef7fbc39728b`); touched all-target
compilation passed under the same explicit solver/service fixture closure. Main subsequently
received the selected native-member, protected-reader, cache-release, prerequisite and FeedFlash
population: 23/23, zero failures/build warnings, complete capture/settlement
(`01a1b371-1d64-75fe-b5f8-29f8588360d3`). This population is separate from the earlier 70 controls.

A whole-member scope can qualify an original trajectory separately from fit optimality;
a sampled point needs its own retained native coordinate authority. Original physical
time, authored target/source and mesh-site correspondence describe points; row ordinals, callback
caches, caller maps and newly hashed time labels cannot supply authority. Missing demanded accuracy
evidence withholds use. Preserve qualified siblings independently of aggregate fit failure.

The first L02 consumer initially supplied an integration fixture to `CaseOperation`, whose actual
contract is an authored solve with SolveSettings; that request correctly refused. Existing typed
`SimulationOperation` executes the original fixture and retains its physical controls. Preserve
these distinct operation owners rather than infer a need for universal dispatch from that caller
mistake. A01 assesses useful discovery/selection improvements against real consumers.

The standalone native trajectory now consumes the same member owner, retaining its original
immutable snapshot, completion, inputs and actual attempt. `qualified_report(&RunResult)` checks
the real producer/snapshot correspondence. The protected
[`trajectory reader`](../../crates/pse-runtime/src/workflow/connected_results/trajectory_permission.rs)
interprets the recorded decision from the same protected completion and `scientific_contexts`;
it checks original run/attempt/member/source/physical/preparation/profile plus output/quantity/unit
and exact sample-time correspondence. It shares existing protected metadata/decoder ownership,
uses generated column references and keeps projected permission roots complete. Full original
coverage is usable, selected subsets partial and missing/foreign/ambiguous facts unknown.
Decoded observations retain no native proof. This does not restore providers, inputs or a point
certificate from historical bytes. The actual aqueous protected/IPC journey passes after producer
drop, with independent raw wire interpretation and native-coordinate mismatch controls.

Native standalone callers can select one authored output at an exact retained physical time
through `ModelingTrajectory::qualified_output_at`. It uses the original joined report and
full native sample coverage, and returns the existing sampled-value row with canonical
quantity/unit. The projected row index follows the native coordinate. Missing, foreign,
refused or passively decoded member proof cannot select a qualified value. Independent
point-product accuracy and derivative permission still need their own original evidence.

The Case entry now identifies the authored case when its integration procedure requires the
Simulation entry and integration controls. Its paired control is in the aqueous protected
journey. Current receiving stops during canonical source publication with `SourceEditUnsettled`
after the driver reports `ConnectionFailed` and an unknown server outcome. Explicit setup
settlement exposes that original error, which was previously masked by fallback fixture drain.
The scientific assertions in this receiving run remain pending while original source publication
is unresolved. The separate standalone sampled-value control passed under native Diffsol.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| A01 | in-progress | Existing declared/prepared/completion owners; first scientific and developer consumers | First passive result and bounded physical inventory consuming agreements are recorded; remaining operation/input/product gaps follow their actual consumers without replacing working admission. |
| A02 | in-progress | A01; actual fit/Study/trajectory product producers | Fit and standalone consumers now share exact sampled-output lookup under original member permission. Verify the current profile contract and finish broader point/product authority and Study receiving. |
| A03 | planned | Selected G/H/I physical consumers | Extend material/coordinate and physical operating-program contracts only at exposed shared gaps; retain explicit unsupported combinations and datum/derivative premises. |
| A04 | in-progress | A01/A03; J mixed experiments | The working tree binds exact authored later intervals to local fit axes and keeps the initial interval as the current default when the field is absent. Finish generation and run mixed steady/transient, replay and wrong-boundary controls. |
| A05 | planned | A02/A04; E/F and selected G–K deliveries | Finish changed consumer population and remove superseded mode/scope interpretations; update enduring contracts through applicable decision routes. |

Packages consume narrow working prerequisites rather than entire companions. In particular,
ordinary steady analysis does not wait for distributed or specialized physics.

## Verification and outcome

The working tree also checks the admitted route and procedure kind before preparation.
Run the relabeling and same-kind initialization controls with A04. Arrow remains active
in the authored fit relation, registry, canonical admission and replay. Current-only
acceptance uses the generated fit relation and original native inputs.

**Tested, selected:** the original fit-member and standalone-member controls passed
in native runtime invocation `01a1bd26-6f64-7207-8da5-08ef8c807c89`, selected with
`scripts/pse test pse-runtime --lib --features native-service,engineering-service,property-coolprop,solver-diffsol,solver-ipopt,solver-kinsol,solver-root-isolation`.
`FitTrajectory::qualified_output_at` retains the original attempt and member with the
sample, including exact original time and authored output identity for two outputs
of the same quantity. Unavailable or ambiguous original products withhold use.
The combined selection returned 33 passes and one aqueous source staging failure
against zero, with 11 build warnings and complete settlement. [Fit sample owner](../../crates/pse-runtime/src/workflow/fitting/trajectory.rs)

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
includes the combined generated world. Its selected fitting/service/result/reader receiving,
also selecting `product-reader` and the product binary/public-reader targets, passed 70/70 with
zero failures/build warnings, complete capture/settlement and no pending native lifetime.
That original population did not include the member extension; the subsequent 23-control main
receive above includes its selected producers, protected reader and lifetime controls. Neither
population receives every A02 consumer.

**Outcome:** the bounded assessment, member and trajectory deliveries above have selected
evidence. Shared operation discovery, remaining point/product and physical contracts and wider
member consumers remain open. The newer admission, schedule and profile changes await
integrated verification.
