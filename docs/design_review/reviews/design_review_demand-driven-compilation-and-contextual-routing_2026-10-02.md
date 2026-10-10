---
title: Demand-driven compilation and contextual solver routing
date: 2026-10-02
tier: design
purpose: target
standard: core-3.3
profile: process-simulator-1.3
baseline: e149ba2308d87ee36812a0bb0744b68670120d17
evidence: Proposed
decision: Accept
---

# Demand-driven compilation and contextual solver routing

**Accept the bounded target at Proposed evidence.** The independent design reviewer found
ADR-0152 and Plan 25l a coherent correction to the preparation, routing and completion
interactions diagnosed by the [full-case review](design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md).
The coordinator publishes that judgment and its bounded argument here. Current implementation
remains **Revise**; no source diagnosis, scientific result or performance limit is erased.

## Scope, evidence and decision owner

This design-tier target review applies Core 3.3, ProcessSimulator 1.3 and the pse-arrow
binding. It examines semantic selection, demanded mathematics, contextual assessment,
execution composition, completion, identity and generated/durable boundaries in proposed
ADR-0152 and Plan 25l. Square simulation, optimization, initialization, fitting, dynamics,
studies, topology, selected observations and failure paths are relevant consumers. Authored
physics, numerical algorithms and supported formulation classes are preservation constraints.
This is not whole-simulator numerical qualification or an exhaustive adapter characterization.

HEAD was `e149ba2308d87ee36812a0bb0744b68670120d17`, plus the newly proposed ADR. Production
was unchanged during review. The reviewer independently inspected the proposed documents,
the prior diagnosis, blueprint §14.1/§14.3/§14.4/§18.7, ADR-0146 and relevant compiler, math,
runtime, routing, dynamics and completion source. The earlier diagnosis's adapter evidence
is reused within its recorded scope. No tests, probes or measurements were run. Source seams
are **Interface-checked**; replacement contracts are **Proposed**.

The [Plan 25 coordinator](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25-design-remediation.md#finding-dispositions) owns
finding dispositions; Plan 25l owns functional packets and 25k owns assembled qualification.
L0 must amend architecture before affected production code. ADR status and decision-PR
acceptance remain separate from this target review.

## Owned operations and composition

| Owner | Consumed meaning and product | Effect/test boundary |
|---|---|---|
| Checking/specialization | Checked revision, physical context, finite bindings, membership and connection closure → semantic inventory | Topology needs no body or derivative preparation |
| Case formulation | Purpose, roles, observations and mandatory closure → original case and selected outputs | Physical/domain/effect obligations survive |
| Compiler/math integrations | Admitted program, ordered outputs/coordinates, order and limits → immutable support/representation products | Existing libraries own mathematical algorithms |
| Shared routing | Intent, class preference, ranks, explicit selection and structural policy → candidate decision | Pure policy over explicit evidence |
| Adapter/settings/representation owners | Actual method, representation, build/runtime observation and guarantees → contextual requirements/refusals | No copied central conditional table or ambient observation |
| Runtime | Finite demands, selected products and snapshot → final readiness and attempt | Existing jobs/flights/retention; worker-owned mutable state |
| Completion | Native termination, actual original assessment, permission and causes → diagnostics | Registry/Rust-generated transport; recorded interpretation preserved |

Source supplies credible seams: `specialize_modeling` already returns semantic specialization;
the existing complete preparation and flow caller instead invoke eager admission. `LocalDemand`
already names body, outputs, coordinates and order, and assembly workers already own mutable
evaluators/providers. These seams support the correction without establishing it as implemented.

### Evidence dependencies

The target correctly rejects a rigid facts-before-preparation pipeline. Incidence, coefficient
extraction, class proof or provider evidence can require finite support products. Pending
evidence returns its dependencies; the consuming decision is reassessed after preparation.
Missing coefficient/convexity evidence does not prove a smooth-only class. Relevant higher-priority
classes must be established or ruled out before selecting a lower class; square Root does not
demand unrelated quadratic extraction. Numerical PSD retains its explicit policy.

This is owned evidence composition, not a numerical fixed-point engine. Evidence progresses
monotonically for one immutable snapshot. Repeated unsatisfied demands/cycles refuse with
attribution; resource failure cannot license truncation or another solver.

Conservative all-branch incidence is distinct from evaluated derivatives. An opaque/value-only
provider may supply dependency evidence without a Jacobian implementation. Complete original
rows/free columns remain; structural rank does not prove numerical rank. Candidate-specific
structural interpretation and final readiness precede execution.

## Scientific preservation and scenarios

| Model element | Physical convention | Preserved validity/authority |
|---|---|---|
| Material ports | Declared mass/molar, indexed species/phase and directed occurrences | Checked conversions, connection closure, composition and applicability; blueprint §8–§9 |
| Temperature/pressure | Point/difference, datum and absolute/gauge remain distinct | Original domains/property envelopes; physical operation owner |
| Energy | Declared extensive/intensive and temporal basis, reference/sign | Independent original/temporal conservation; authored contributions |
| Implicit/property selection | Relation versus operational function and phase bounds | Branch, graph fidelity and derivative evidence; existing implicit owners |
| Indexed/temporal state | Ordered axes, finite membership, initial condition versus specification/guess | Mass, event/reset, initialization and sensitivity contracts; existing dynamic owners |

Selected closure includes equations, objectives, required observations, checks, domains,
applicability, effects and transitive providers. Constant normalization or zero derivatives
cannot erase mandatory obligations; Second-only conditions do not become Value conditions.
Topology is neither incidence nor solve order. Original physical tolerances, scaling and
qualification remain independent of native success. No changed bound, dummy objective,
unrequested approximation or expanded incumbent eligibility is selected.

| Scenario (prior S01–S06) | Distinguishing acceptance |
|---|---|
| Topology and selected Value/First | No executable admission for topology, no numerical derivatives for Value, no Second for First; invalid physical/domain obligations still refuse |
| Boxed square C2 versus C1 | Missing Second artifacts differs from scientific incapability; current ranks and explicit selection survive |
| Pending class/structure | Obtain demanded proof rather than interpreting absence negatively; conservative provider incidence does not promise derivatives |
| Repeated indexed model/edit | Alias maps, fresh attribution, clean/incremental equality and value-only reuse survive |
| Dynamic method substitution | Auto assesses complete contract; incompatible explicit method refuses; events/trials/sensitivity not downgraded |
| Policy/failure locally | Actual typed inputs without native/storage startup; complete candidate/quality/cause/status matrix |
| Boundary evolution | New vocabulary generated from its owner; old recorded domains and digests retain meaning |

Ordinary model extension should remain with authored definitions/specialized behavior. Mechanism
substitution belongs to its integration owner and capability contract. Proposed locality must
be checked during implementation and assembled review, not inferred from this document.

## Architectural foundations and gates

These judgments apply to the bounded **Proposed target**, not its current realization.

| Foundation | Verdict and basis |
|---|---|
| AP-01 | Satisfied: semantic selection, demand, policy, contextual conditions and completion have coherent owners |
| AP-02 | Satisfied: missing evidence, missing artifacts, refusal/readiness and evolution retain distinct contracts |
| AP-03 | Satisfied: finite demands compose existing operations without copied end-to-end workflows |
| AP-04 | Satisfied: demand, availability, structure, readiness, assessment and native outcome govern operations |
| AP-05 | Satisfied: dependencies, budgets, maps, snapshots and transitions are explicit |
| AP-06 | Satisfied: pure policy has explicit observations; weaker consumers avoid unrelated infrastructure |

| Gate | Target judgment and limit |
|---|---|
| G1 Authority | Pass, Proposed: existing semantic/library/adapter authorities and owned projections |
| G2 Fidelity | Pass, Proposed: incidence/derivative and absent/violating assessment distinctions |
| G3 Validity | Pass, Proposed: selected obligations and contextual structure/readiness before execution |
| G4 Hidden behavior | Pass, Proposed: explicit immutable runtime observations |
| G5 Recovery | Pass, Proposed: immutable upgrades, retryable transient refusal and existing lifetime owners |
| G6 Transformation/reuse | Pass, Proposed: complete maps/keys, original checks and preserving recorded contracts |
| G7 Truthful capability | Pass, Proposed: static inventory differs from contextual readiness; no failure-driven fallback |
| G8 Library leverage | Pass in inspected target: libraries retain mathematical and native algorithms |
| G9 Fitness | Pass, Proposed: all six applicable foundations satisfied within boundary |
| PS-G1 Physical consistency | Pass, Proposed preservation contract; full-case numerical fidelity unqualified |
| PS-G2 Well-posedness | Pass, Proposed: complete incidence and candidate structural admission; topology not solve order |
| PS-G3 Numerical integrity | Pass, Proposed: demanded derivatives/methods, independent original checks and truthful completion |

## Failure truth, identity, alternatives and findings

Completion correction composes with preparation/routing correction without weakening use.
Missing quality gains its own refusal; evaluated violations remain Infeasible and absence of
a candidate remains NoCandidate. Callback/validation/evaluation causes retain stage/identity.
Exhaustive projection retains precise native limits, cancellation, numerical and inconclusive
meanings. Contradicted infeasibility remains inconclusive. Permitted feasible incumbents remain
accepted, explicitly non-optimal and free of rejection diagnostics.

Auto reconsideration requires newly established scientific/representation incompatibility
against the same snapshot/ranks. Resource, cancellation, infrastructure and native attempt
failures stop the request. Final preflight verifies readiness, not the first contextual check.

Keys cover selected outputs/coordinates/order, consumed physical/provider dependencies,
representation and relevant policies/build contracts. Runtime observations affect only products
that consume them. Values and occurrence attribution remain separate. Stronger failures cannot
mutate weaker products. ADR-0146's recorded interpretation, directional projection, exact write
admission and explicit preserving migration govern boundary evolution; internal support need
not become public, and historical frames are not recomputed.

Tightening existing owners is the simplest viable choice. Larger budgets or explicit fixture
routes do not correct the causes; maximum-candidate preparation forces unnecessary work.
A DSL/universal IR/solver framework adds unsupported interpretation/maintenance cost. Libraries
remain numerical authorities; existing Salsa/cache/flight/accounting mechanisms absorb the
additional immutable products rather than creating another cache authority.

**No new blocking target finding.** Prior F01–F03 retain their identifiers and separate closure
obligations. Target acceptance does not resolve them. No MUST waiver or SHOULD exception.

## Authority route, verification and decision

L0 records ADR-0152 and amends §14.1/§14.3/§14.4, §18.7 and affected identity/boundary owners
through the design route before governed code. In particular, §18.7 must replace static-only
eligibility with composition of shared policy and owned contextual conditions, retaining one
static inventory declaration. The coordinator has recorded the proposed supplements and
blueprint revision 97 through the authorized route.

Plan 25l owns focused controls, consumer migration and deletion. Plan 25k owns scientific
journeys, lifecycle/reuse/resource evidence, dev-profile measurements and final conformance.
Historical CSTR receipts and PR/PFR refusals retain their earlier limits. No Tested, Measured
or Formally established replacement claim is made by this review.

**Behavioral/semantic adequacy: Accept at Proposed evidence. Architectural fitness: Accept at
Proposed evidence. Overall: Accept the bounded target.** No consequential target choice remains
unresolved. Local API names/layout are implementation discretion. Authorized implementation
may proceed after the recorded L0 amendments; replacement acceptance requires the named evidence.
