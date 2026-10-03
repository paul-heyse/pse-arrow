---
title: "25l: Demand-driven flowsheet compilation and contextual solver routing"
status: draft
date: 2026-10-02
adrs: []
review_sources: [docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md]
scenario_sources: [docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md#s01, docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md#s02, docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md#s03, docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md#s04, docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md#s05, docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md#s06]
---

# 25l: Demand-driven flowsheet compilation and contextual solver routing

## Purpose, baseline and ownership

Make full case compilation compose domain-owned operations: select checked process meaning,
formulate the case, obtain the evidence a decision consumes, assess backend requirements,
prepare the selected products, then execute and qualify in original physical terms. A topology
request must not depend on executable arithmetic; a Value or First request must not pay for
unrequested Second support; automatic routing must be able to request available mathematics
rather than treating its current preparation state as scientific incapability.

This is the functional follow-up to the
[flowsheet compilation and solver routing review](../design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md).
Its F01/F02/F03 references are source-qualified and distinct from the older findings with the
same numbers. The [Plan 25 coordinator](25-design-remediation.md#finding-dispositions) remains
the single disposition owner. This document owns the target and packet progress;
[25k](25k-integrated-qualification-and-closure.md) owns assembled qualification and measurements.
The alphabetic suffix does not imply execution after qualification: these functional changes
precede resuming the paused K3 campaign.

**Implemented baseline:** `ec45d4c8640f1dcbc04e0eb8a552e146f4d3ac39`, with the preserved
uncommitted review and 25k review-boundary checkpoint. Plans 25a–25j retain their completed
scope and historical evidence. The review diagnoses implemented defects; its remedy and this
target are **Proposed**, not accepted architecture or qualified production behavior. No new
product tests or performance campaigns were run while authoring this plan.

Apply the selected Core 3.3 and ProcessSimulator 1.3 standard through the repository binding.
Architecture remains owned by blueprint §7/§14, §15/§18, §19 and §23.2. This plan schedules
their decision-route amendments; it does not amend them in passing. Plan authoring authorizes
no production implementation, commit, push or publication.

## Assessment of the affected foundations

The current semantic model is a suitable foundation. Checked revisions and `SpecializedModel`
already retain physical/indexed ownership, ports, connection occurrences, formulations and
original obligations. `FlowGraph` correctly distinguishes physical topology from equation
incidence and solve order. Complete original rows/free coordinates, structural witnesses,
conditional-unit locality and independent physical checks must survive the correction.

The preparation boundary needs to change. `ModelingPackage::prepare_flow` currently calls full
preparation, and compiler modeling preparation combines specialization, broad projection and
body admission. `BodyBuilder::prepare` constructs both derivative support orders before a
consumer selects its demand. Existing selected stages and `CasePlan::LocalDemand` already
express useful output, coordinate and order distinctions, but applying them after eager body
admission cannot establish selective preparation. Reuse those meanings at their producing
owners; do not build another expression interpreter or generic compiler instruction model.

Focused independent design advice confirmed these seams and identified class discovery,
per-candidate structural interpretation and selected-output availability as additional joints
to make explicit. Its source conclusions were reconciled here; it supplies no separate formal
verdict or runtime qualification.

The routing boundary also needs to change. `ProblemFacts` already separates mathematically
available from prepared derivatives; `routing::Decision` retains admission facts even when
there is no attempt. Preserve both. The current comparison against their minimum, speculative
`SolverProfile::derivative_order`, and contextual checks after generic selection prevent those
foundations from composing. Structural analysis currently consumes `CasePlan` support, so a
strict one-pass facts-before-preparation pipeline would be an incorrect replacement.

The adapter seam is reusable. Each adapter already owns settings, representations, library
integration and native execution. Contextual assessment must consume those owners, rather
than copy their conditions into a second central capability table. Symbolica/Numerica, faer,
the existing structural libraries and native solvers continue to own numerical algorithms.
The review's pinned characterization supplies the baseline; verify a library contract only
when an implementation choice depends on a gap or changed version.

Completion has an independent, narrower defect. An absent `quality` currently produces an
`Infeasible` refusal, and several native limits fall through to `TrialRejected`. Candidate
use is already safely refused. Correct the meaning without weakening usability, seed policy,
independent qualification or feasible-incumbent rules.

### Choices and alternatives

Select staged demand and contextual assessment through existing owners. Compared with preparing
every candidate's maximum requirements, it admits lawful weaker consumers without requiring
unrelated Second/factorable work. Compared with larger budgets or explicit fixture backends,
it addresses the causal preparation and routing boundaries. Its cost is additional immutable
products, complete keys and explicit readiness transitions, handled by the existing compiler
and runtime ownership mechanisms.

A new DSL, universal IR, solver algorithm or generic workflow framework is not selected.
Reopen that choice only if an actual supported case cannot be represented by the existing
semantic operations after these boundaries are separated. No new crate, backend, dynamic
formulation class or approximate mathematical fallback is required. Existing library capability
eligibility and technical dependency selection remain in force.

Selective work is expected to reduce unnecessary construction; this is a **Proposed** benefit.
Requested exact derivatives can remain expensive and can lawfully exhaust finite limits.
Neither PR/PFR's historical refusals nor immutable fact sharing isolate a measured speedup.

## Target contracts and interactions

### Semantic admission and selected case meaning

Expose the checked, finite specialization product independently of executable body admission.
It consumes the immutable revision, physical context, root/instance, static/indexed bindings,
requested semantic membership and finite limits. It retains actual member identities, values,
connections, original declarations, checks and attribution. Physical/type errors and invalid
selected connection closure remain refusals. No numerical evaluator, derivative support or
solver initialization is needed to inspect its topology.

Case formulation consumes that product plus declared purpose, fixed/free roles, objectives,
formulation selections and observations. Select numerical outputs before admitting their
executable bodies. The selected closure includes equations, objectives, required observations,
original checks, property/applicability obligations, effect dependencies and nested providers.
Retained source declarations need not all become observable identity outputs. Nonexecuting
data remains explicitly retained under admission closure; arbitrary unsupported selected terms
cannot be silently dropped. Each selected body uses existing checked physical occurrences.

The same original case meaning feeds callback, coefficient, cone and factorable realizations.
Representation transformations preserve original-space qualification. A density relation is
not interchangeable with an operational density selector; relaxed graph export remains
explicitly permitted or refused. Conditional solve factories with method-specific maps retain
their actual contract, even when the outer case can use another backend.

### Demand-indexed mathematical products

Separate the immutable value/control/domain/effect program from requested derivative support.
A support request names its admitted body, selected outputs, differentiation coordinates,
order and finite construction policy. Reuse the existing local-demand meaning; local type
names and module layout are implementation choices. Domain/effect dependencies are retained
even when normalization yields a constant or zero derivative. Guard order remains meaningful:
a Second-only validity requirement is not imposed on Value, while a value-domain guard survives.

Value prepares no derivative support. First constructs only its demanded first-order closure;
Second adds only its demanded second-order closure. Structural incidence may request a
conservative First support product without evaluators. This means potential all-branch
equation/coordinate dependence, not proof that a provider can numerically evaluate First
derivatives. It may retain dependency evidence for an opaque/value-only provider without
inventing a derivative implementation. That product retains complete original
rows/free columns and conservative dependencies; it does not certify numerical rank. A
coefficient/class or provider proof may request further support explicitly. Do not derive
scientific availability from which support happened to be constructed first. Availability is
computed for selected outputs and their mandatory closure; an unrelated nonsmooth output or
unused provider must not lower that consumer's ceiling.

Output and coordinate selections are deterministic and checked, including aliases and repeated
instances. Stable body-local positions are translated through explicit selected maps; filtering
must not reinterpret an old index as a different member. The existing total checked-member
binding supplies actual occurrence attribution. Clean and incremental construction must agree.

Stronger requests produce immutable additional products and cannot mutate weaker products held
by callers. Failed/cancelled support construction is retryable, does not poison weaker entries,
and is never memoized as mathematical incapability. Check finite operation/storage allowances
before protected allocation; splitting stages must not reset or multiply the request's budget.
Keep construction, support, evaluator and worker charges attributable to their actual owners.

Preserve semantic body identity where its meaning is unchanged; use distinct versioned keys
for changed preimages. Keys cover consumed semantic/physical/provider dependencies, outputs,
coordinates, order, representation, relevant policy and build/evaluator contracts. Value-only
rebinding and per-instance attribution remain separate. Runtime observations affect assessment
identity where consumed; unrelated runtime facts must not force rebuilding symbolic bodies.
Existing Salsa/service retention, single-flight completion and allocation leases remain the
only cache/accounting mechanisms. Historical frames and digest bytes are never recomputed.

### Contextual candidate assessment and readiness

The planning input is the selected case snapshot, intent/selection, controls/settings, requested
guarantees, scientific evidence, available prepared products, and an explicit immutable
build/runtime observation. Environment discovery stays at the composition boundary; a pure
assessment does not read ambient process settings or start native/storage services.

Class discovery has its own evidence dependencies. An absent coefficient snapshot or
unconstructed Hessian is not proof that a case is non-coefficient or nonquadratic. Before
selecting a lower-priority class, establish or rule out the higher-priority classes relevant
to that intent through existing library-owned shape/coefficient/convexity evidence. Request
missing proof or extraction only where that decision needs it; Root's established square
class does not force unrelated quadratic extraction. Numerical PSD evidence still requires
the existing explicit policy. Resource failure while obtaining required class evidence is
a preparation failure, not permission to fall through to another class.

Assessment distinguishes these meanings:

| Product | Meaning and downstream action |
|---|---|
| Contextual refusal | The actual case, settings, representation or observed runtime cannot meet a requirement; retain typed causes and all known applicable refusals |
| Pending evidence | A structural, scientific or representation decision lacks evidence; return its finite dependencies without asserting eligibility or incapability |
| Supported candidate with pending artifacts | Evidence establishes representability; name the required kernels/products not yet ready |
| Ready candidate | Contextual conditions, structural admission and demanded artifacts are established; execution may consume its admitted representation |

These meanings can extend existing assessment/decision products; they do not require a new
public service or a type for every row. Generic class/bounds/intent rules stay with routing;
representation and settings conditions stay with their existing owners. Interpret the shared
structural witness using each candidate's actual policy before deeming that candidate
suitable; preserve Root's overriding square requirements. Final preflight validates the
ready product and snapshot again rather than being the first contextual assessment. Runtime publishes
static inventory and contextual assessment as distinct facts. Any exposed states/reasons are
generated from their Rust/registry owner, not inferred from prose inventory fields.

Preserve algebraic class order, automatic ranks, explicit-only capabilities, lexicographic
policy and authored exact/limited-memory/Gauss–Newton choices. A scientifically supported
candidate requiring Second is considered before a lower-ranked already-prepared candidate;
First preparation is not a reason to demote it. Explicit selection assesses only the requested
backend for selection and never substitutes another. Constant cases retain original checks
and structural/constraint-form admission without fabricating a native attempt.

Prepare unresolved decision evidence as demanded, reassess its consumers, then apply selection
to candidates whose required evidence is established. Prepare the selected artifact demand
and establish final readiness. Evidence sets progress monotonically for one immutable case
snapshot; detect repeated unsatisfied requests/cycles and refuse with their dependency cause.
When candidate-specific preparation establishes scientific or representation incompatibility for that candidate, automatic
selection may reassess remaining candidates using the same ranking and snapshot, retaining the
new reason. Resource, cancellation, infrastructure and native attempt failures stop that request;
they do not trigger backend fallback. No attempt starts before final structural/readiness admission.

Optional post-solve analysis remains separate from mandatory solver inputs. A valid base solve
can retain explicitly unavailable optional sensitivities; a requested mandatory dynamic mode
or exact transient Hessian must meet its own method/kernel requirements. Inner provider demand
propagates only from selected consumers through the existing provider dependency order.

### Contextual responsibilities across current adapters

All nine exposed adapters participate; this is migration scope, not a new numerical capability
claim. Lift existing conditions to planning without weakening final admission:

| Owner | Conditions that planning must consume |
|---|---|
| Ipopt / POUNCE NLP | Authored Hessian mode and resulting order; general bounds; actual method requirements; selected linear solver/build/thread prerequisites; requested start/reuse guarantees |
| KINSOL | Square contract, one-sided shifted sign bounds versus boxes, Jacobian/JVP demand and settings-specific splitting/map restrictions |
| HiGHS | Admitted coefficients, supported continuous convex QP versus MILP classes, native forms, lexicographic route and actual method's interruption/start contract |
| Clarabel / POUNCE-convex | Admitted cone/coefficient evidence, cone subtype/build support, existing automatic-class and explicit-only restrictions, update/start restrictions and cancellation/deadline semantics |
| SCIP | Factorable export and actual native-handler admission, fidelity/global finite-domain requirements, exact-mode/settings restrictions and reoptimization contract |
| Diffsol / IDAS | Actual method, mass/algebraic structure, initial consistency, trial policy, directional events, reset/quadrature rules, requested sensitivity mode and exact transient Hessian requirements |

For dynamic Auto, assess the requested contract against linked Diffsol and IDAS methods before
selection. Prefer the existing Diffsol default when it meets mandatory requirements; choose
IDAS when those requirements exclude Diffsol and IDAS can meet them. Keep explicit method
selection strict. Do not silently remove events, change trial policy or downgrade a requested
sensitivity. Optional analysis withholding does not falsely make a supported base route absent.

External starts, internal retained state, allocation/factor/search reuse and batch execution
remain distinct. Retain existing typed contracts. Make cancellation/reuse/diagnostic metadata
typed where a caller actually uses it to decide readiness or execution permission; otherwise
leave informational prose as presentation. No consumer may parse those descriptions as policy.

### Completion and diagnostic meaning

Add a registry-owned candidate refusal for unavailable original-space feasibility assessment.
`None` quality yields that reason, while an evaluated violating quality yields `Infeasible`.
No candidate independently yields `NoCandidate`; multiple true reasons may coexist. Callback
or validation failure retains its typed cause and failed stage. None of these permits result
use or invents a feasibility conclusion.

One completion-owned projection consumes native termination, original assessment, candidate
decision and retained causes. It supplies public diagnostic classification; adapters continue
to own native status interpretation. Preserve detailed statuses instead of reconstructing
them from broad public classes. Use an exhaustive mapping with these defaults:

| Native/assessment evidence | Public diagnostic when the result is refused |
|---|---|
| Contradicted infeasibility evidence | Inconclusive, retaining its witness |
| Callback/validation/evaluation cause | Cause-derived class and stage; no replacement by a fabricated infeasibility |
| Cancelled | Cancelled |
| Time, iteration, node, solution, objective or generic limit; ResourceExhausted | ResourceLimit, retaining the precise native category |
| Numerical | Numerical |
| Inconclusive | Inconclusive |
| Panic / Invalid without a more specific retained cause | Internal / InvalidModel respectively |
| Native infeasible, unbounded, infeasible-or-unbounded; rejected Success/Acceptable/FeasibleOnly | TrialRejected with the precise native conclusion and actual qualification reasons |

An accepted feasible limit incumbent produces no rejection diagnostic and retains its
non-optimal qualifier. This projection does not expand incumbent eligibility for limit
categories currently refused by `native_use`. A native infeasibility conclusion is retained
as native evidence; it does not manufacture an evaluated original-space quality record.
Retry policy continues to require the actual cause and known-safe effect boundary.

## Decision route and execution packets

L0 records the new staging/admission contract before production changes. Allocate a new ADR
through the repository route covering staged compilation, contextual readiness, changed current
identity preimages and generated assessment distinctions. Hashing/Python-boundary changes need
the ADR plus a bounded target review; the current Revise review is diagnosis, not acceptance
of the new implemented design. Preserve proposed ADR-0144/0145/0146/0150/0151 and immutable
accepted records. Amend blueprint §14.1/§14.3/§14.4, §18.7 and relevant §19/§23.2 owners through
the design route and revision row. Add the allocated ADR to this front matter when it exists.

Explicitly amend §18.7's current rule that generic capability-record admission alone governs
eligibility and adapters cannot contribute contextual assessment. The replacement composes
shared generic policy with owned settings/representation conditions, with static inventory
still derived from the one adapter record. This authority change is a prerequisite, not an
implementation detail to postpone to final qualification.

For each changed public/durable operation, migrate its producer, generated schema/codec and
consumers in the same packet. Use ADR-0146's exact recorded-contract and directional admission
operations. Historical artifacts open with recorded vocabularies; new values require a capable
consumer and current writer, never transform-on-open. An internal-only support product need
not become persisted or public. If stored layout changes are needed, the owning packet includes
an explicit preserving migration and its interrupted-prefix controls before current writers open.

| Packet | Prerequisite and editing responsibility | Delivered behavior and focused acceptance | Status |
|---|---|---|---|
| <a id="l0"></a>L0 Decision and shared contracts | This target; coordinator owns ADR/review and shared contracts | Record the contract/readiness/identity/public-evolution decision and required architecture route before affected code; target review settles consequential objections | planned |
| <a id="l1"></a>L1 Semantic selection and executable closure | L0; modeling/compiler specialization, runtime package and flow/inspection consumers | Topology consumes checked specialization without executable admission; numerical projection admits selected output/dependency closure; migrate callers and delete broad eager preparation bridges; controls for topology, unrelated observations and mandatory physical/domain obligations | planned |
| <a id="l2"></a>L2 Demanded support and owned products | L0 and L1 selected closure for modeling integration; math/compiler body, assembly and runtime product/retention owners | Value/First/Second support separation, conservative structural evidence, complete selection maps/keys and finite budgets; migrate body/view/artifact consumers and remove eager Second analysis; stronger failure preserves weaker products, aliases and clean/incremental equivalence | planned |
| <a id="l3"></a>L3 Contextual candidate assessment | L0 shared contract; native routing, representation, settings and dynamic owners | Typed refusal/pending evidence/pending artifacts/readiness across nine adapters, explicit snapshot and unchanged ranks; lift contextual checks without copying policy; isolated bounded-C2/C1, method/build/cone/factorable/event controls and generated assessment projections | planned |
| <a id="l4"></a>L4 Compose preparation and execution | Working L1/L2/L3 products; runtime math and affected workflow owners | Deterministic evidence/demand composition, selected preparation, final admission and consumer cutover; remove speculative Auto order and late-only selection paths; migrate inner providers, direct/declared solve, initialization, fitting, dynamic/shooting, studies, inspection/conformance and Python/durable adapters | planned |
| <a id="l5"></a>L5 Truthful completion and projections | L0 boundary decision; completion/diagnostics/schema and affected transport/storage owners | Unavailable assessment refusal and exhaustive shared termination projection, unchanged result/seed/incumbent policy; regenerate and migrate current readers/writers; remove missing-as-infeasible and wildcard resource classification; isolated status/quality/cause matrix and historical-contract controls | planned |
| <a id="l6"></a>L6 Functional handoff | L1–L5 complete, required decisions satisfied | All migrated consumers and replacement deletions checked; revealing fixtures registered in existing harnesses, focused evidence and exclusions recorded; hand off new static/integrated/measurement scope to 25k, without running a second campaign here | planned |

One useful order is L0 → L1 → L2 → L3 → L4, with L5 available after L0, then L6.
L3 contract/adaptor work can start after L0 without waiting for all L2 implementation;
L4 integration requires working products, not merely an agreed interface. Mathematical support
work can start after L0 while L1's consumer migration proceeds, with their shared compiler
surfaces assigned to one writer. These are dependencies below whole-document boundaries, not
synchronization barriers. Runtime evidence requests are not an implementation dependency cycle.

The coordinator owns design, shared declarations and integration. Executors may choose local
organization within their packet; split shared compiler/runtime edits only after assigning
ownership. Preserve concurrent changes in the existing checkout. No temporary old/new API pair
or alternate production engine survives a packet. Retain independent scientific oracles and
historical contract fixtures for their continuing meaning, not deleted-path coverage.

## Verification and acceptance

**Proposed:** the controls below refine the source review's S01–S06; they are not a second
scenario registry or claims of executed tests. Every functional packet uses recipe-owned
compile checks and focused tests with explicit force-validation. Use `just check-package`,
`just check` for cross-crate changes and `just unit-package <pkg> <filter>` for actual isolated
units. Register broader fixtures during implementation and select their existing integration
harness in K3; do not disguise a native/storage journey as a unit test. Regenerate changed
declarations through `just codegen`, never edit generated output.

| Scenario/control | Required observation |
|---|---|
| S01/S03/S05, same definition at topology/Value/First/Second | Counters show no executable admission for topology, no derivatives for Value, no Second support for First; selected Second can exceed its explicit allowance without invalidating weaker products |
| S01/S03, output/dependency selection | An unrelated expensive observation is not admitted; selected domain/effect/applicability/closure checks survive zero derivatives, normalization and branch selection; invalid physical declarations still refuse |
| S02/S05, boxed square C2 and C1 | First-prepared C2 reports exact NLP artifacts pending and selects the existing lawful rank; genuine C1 refuses exact Second; unused provider/nonsmooth output does not lower selected availability; explicit incompatible/unlinked selection refuses; no dummy objective or changed bounds |
| S02/S05, pending class/structural evidence | Missing coefficient/convexity products request evidence rather than implying smooth-only class; a candidate's structural policy is assessed before suitability; conservative provider incidence does not promise evaluated derivatives |
| S02/S04/S05, contextual routes | Method/build threads, cone subtype, factorable fidelity/handlers and event/trial/mass/initialization requirements distinguish readiness before attempts; directional Auto uses linked lawful IDAS, explicit incompatible Diffsol refuses |
| S03/S04, providers and scientific meaning | Nested demand is selected/transitive; method-specific inner factory contracts remain; PR operational/relational choices are distinct; optional response unavailability does not invalidate an otherwise accepted base solve |
| S03, reuse and attribution | Multiple PFR meshes/instances reuse equal body mathematics; selected maps remain correct; edit/rebind and clean/incremental products agree; old results retain actual member attribution; changed consumed context invalidates reuse |
| S05/S06, assessment/quality/status matrix | No candidate, unavailable quality, evaluated infeasibility, evaluation/validation failure, contradiction and every native category retain correct reasons; feasible permitted limit incumbents remain usable and explicitly non-optimal |
| S04/S06, failure and boundaries | Resource/cancel/infrastructure/native failures never trigger an unrequested backend retry; failed stronger preparation remains retryable; generated Python/durable projection and old recorded contracts preserve distinctions |

For full scientific acceptance, K3 executes bounded CSTR Auto and explicit controls, PR
relational and operational controls, PFR indexed multi-mesh controls, and the affected connected,
initialization, fitting, event/shooting, study and durable journeys. Keep authored bounds,
applicability permissions, tolerances and independent original-space/physical oracles. Classify
a remaining required-case failure by its actual stage and repair it at its owner; increasing
allowances or forcing fixtures to a backend does not close these architectural obligations.

Once L6 establishes functional readiness, 25k refreshes the static gate for changed source,
then resumes K3 and K4, with K5 assessing the assembled target. Earlier K1/K2 receipts remain
historical rather than being relabeled as evidence for this new source. Use the existing
memory-capped Linux recipes and selected dev profile. Repair/rerun failed scopes and repeat
other scopes only when changes can invalidate them; record composite results against zero.

K4 owns actual body/support/artifact construction counts, A/B/A and durable-worker reuse,
retention/escaped-allocation and permit lifetimes, cold/warm preparation distributions and
resource observations. Extend its existing workloads to distinguish Value/First/Second and
selected output closure. No speedup threshold is invented, and RSS is not substituted for
allocation ownership. Optional broader scaling or release-profile timings remain limits;
required successful scientific journeys and reuse/resource controls do not become optional.

F01 closes only when all three preparation separations and mandatory-obligation preservation
are established. F02 closes only when contextual selection, demanded readiness, affected
consumers and the required scientific journeys agree. F03 closes when the shared semantic
projection and generated/recorded boundaries preserve the complete distinction matrix.
The coordinator links packet and 25k evidence before marking findings resolved. No local pass
qualifies an unexercised backend or the enclosing simulator.

## Current state and next action

Target authored; production unchanged. No consequential target choice remains deferred.
Local representation/API names remain implementation discretion. Start L0 after production
execution is authorized, then implement the dependency-ordered functional packets. Qualification
remains paused until L6 readiness; the original review retains its scoped Revise judgment.

## Outcome (recorded after implementation)

### What was built

Not implemented. Record actual mechanisms, deletions, commands, scope and evidence labels here.

### A mistake made and corrected

Record a correction encountered during execution.

### Deviations from the plan, deliberate

None recorded. Route decision changes through their owning ADR/review before implementation.
