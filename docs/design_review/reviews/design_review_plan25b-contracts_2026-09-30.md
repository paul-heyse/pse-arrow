---
title: "Plan 25b scientific admission and applicability contracts"
date: 2026-09-30
standard: core-3.3/process-simulator-1.3
tier: design
purpose: target
evidence: Proposed
decision: Accept
---

# Plan 25b contract review

## 1. Scope, drivers and coverage

This independent review assesses the **Proposed** target in [Plan 25b](../../plans/25b-scientific-knowledge-and-applicability.md), [ADR-0140](../../adr/0140-scientific-knowledge-admission.md) and [ADR-0141](../../adr/0141-applicability-evidence-and-permissions.md), before implementation. The selected standard is core 3.3 with process-simulator 1.3 and the pse-arrow binding. The boundary is B1–B4: scientific composition evidence, derived reaction/material admission, parameter-record selection/coherence, and applicability evidence with named record/family permissions.

**Architectural fitness: satisfied for the proposed target. Semantic adequacy: satisfied at document-contract evidence. Overall decision: Accept at Proposed evidence.** This accepts the contracts and their implementation direction; it does not accept the current implementation or establish numerical/scientific qualification.

Baseline: existing `main` checkout at `332612064d9120dee250d2dd7b8905f087fc9b69`, with proposed ADRs and concurrent uncommitted Plan 25a/docs changes. The final reviewed Plan 25b lists ADR-0140/0141 and assigns B4 the low-level data-use gate, with E3 later qualifying results. The earlier sentence assigning permission application to 25e was corrected before this verdict.

Inspected owners and adjacent consumers: authored `chemistry.pse`, `properties.pse`, `interactions.pse`, `reaction-forms.pse`, `nrtl.pse`, `pcsaft-parameters.pse`, `reactions.pse`, `reactors.pse`, `control-volumes.pse` and `costing.pse`; modeling `envelope.rs` and `specialize/envelopes.rs`; compiler `typed_math.rs`; math `guarded.rs` and relevant execution interfaces; registry `catalog/modeling.rs`; ADR-0127 and blueprint §9, §14.3, §19.5 and §20.5.

The relevant workloads are scientific extension, mixed-source model composition, out-of-region evaluation, local admission/policy tests and edit/reprepare after a semantic selection change. CSTR/PFR source construction and selected property/costing evaluation are in scope. Square, optimization and dynamic solver algorithms, structural analysis, fitting execution and final result qualification are not reassessed. Plan 24 database/readers/assembly and production snapshot bridge, Plan 25e qualification, Plan 25g durable migration and whole-product acceptance are excluded. No tests, builds, benchmarks or static checks were run.

Variation axes are incomplete catalog knowledge, reaction/material membership and extent normalization, independent versus jointly fitted records, asymmetric versus symmetric subjects, and source evidence versus consuming permission. These distinctions justify the new admission contracts; they do not justify a second scientific kernel or universal ontology.

## 2. Decomposition, ownership and dependencies

| Responsibility | Hidden decision / exposed contract | Dependency and effect ownership | Local verification setup |
|---|---|---|---|
| Authored chemistry | Composition completeness, charge evidence, reaction coefficients and extent convention | Chemical declarations remain under their existing physical/domain owners; no store dependency | Small species/reaction declarations with explicit evidence |
| Reaction/material admission | Complete participants, supported material subjects, immutable derived source matrix | Consumes authored chemistry and material binding; reactors consume the admitted projection | Missing-product/inert/normalization controls without a native solve |
| Parameter records and selection | Parameterization/family/subjects/variant, orientation, dependencies and atomic-fit membership | Banks own values; selection references values and closes declared scientific obligations | Ternary and joint-fit fixtures using existing records |
| Generic applicability | Claim scope, region basis, union/dependency semantics and attributed observations | Modeling owns admitted meaning; compiler/math preserve demands through numerical lowering | Unknown/outside/hard-domain and active-branch controls |
| Binding policy | Independent named allow-unknown and allow-extrapolation permissions | Binding owns authorization and lineage; evaluation consults it without mutating evidence | A second unrelated selected record must remain refused |
| Generated transport | Mechanical representation of observations and policy references | Registry owns schema; generators emit boundary types; E3 later interprets qualification | New-state round trips and identity distinction controls |

The composition root is the admitted model/binding selection. Scientific data, permission, evaluation observation and final qualification have separate responsibilities. Ordinary new records and claims extend authored owners; no species/method switch in Rust or database bootstrap is part of admission.

## 3. Contracts, authority and constraints

| Operation / phenomenon | Authority and consumer obligation | Failure / evolution |
|---|---|---|
| Conserved composition | A complete sparse composition supplies omitted zeros; complete-empty is explicit; unknown does not establish conservation | Obtain complete participant evidence before summation; unresolved evidence differs from established imbalance; atomic weights only govern molar-mass derivation |
| Reaction projection | Authoritative coefficients and explicit extent normalization mechanically determine the immutable material projection | Every nonzero participant must be represented, inert extras receive zero, incompatible rate/heat conventions refuse; explicit conserved translation may change species basis |
| Parameter-record selection | Parameterization, contract family, subjects and variant identify scientific records independently of publication provenance | Missing required records, conflicting variants and incomplete dependencies refuse; phase belongs only to relevant shapes; symmetric canonicalization remains the existing declared rule |
| Fit coherence | Dependency edges, atomic-fit membership and derivation lineage have different meanings | Close obligations over the selected model/subjects/domain; only declared subsystem projections may split atomic fits; independent records can mix |
| Predictive absence | Stored/fitted zero, absent required pair and explicitly selected predictive zero are separate products | Predictive products retain rule identity and inputs instead of inventing fitted records |
| Applicability evaluation | Claims declare Region(predicate, basis), Unrestricted or Unknown(reason), with owner/scope/evidence | Missing claims never imply unrestricted use; required dependencies retain their observations; alternatives union only by declaration |
| Data use | Named permissions default false and independently authorize unknown evidence or extrapolation | B4 enforces use; mathematical and hard model-domain failures always refuse; inherited resolution cannot widen the declared target |
| Boundary identity | Claims, closure, conventions and selected permissions contribute to admission/preparation identity | New frames/relation revisions prevent reinterpretation; registry generation replaces the old transport, without a compatibility path |

The keyless common parameter carrier with separate complete keyed pure, phase-specific and pair shapes avoids an optional-key or vapor-sentinel convention. Family denotes the parameter contract, so compatible numeric dimensions do not establish scientific interchangeability. Neither provenance nor equal coefficients substitutes for identity.

The existing elemental control-volume `atoms` callback is also affected: ADR-0140 requires those projections to consume admitted composition authority. Removing only `RateLaw.stoichiometry` would leave the target incomplete. This is an existing target obligation, not a recommendation to introduce another coefficient authority.

### Physical semantics

| Element | Dimension / basis | Convention | Applicability / authority |
|---|---|---|---|
| Formula and charge | Element counts and charge numbers are dimensionless, with different physical meanings | Composition completeness and charge evidence are independent | Authored chemistry and claim-specific conserved admission |
| Reaction coefficients | Dimensionless species source per reaction extent | Signed coefficients retain the explicitly selected extent normalization | Reaction declaration; immutable material projection |
| Rate and extent | Volume-based extent rate in mol/(m³·s); volume converts to extent/time in mol/s | Rate and heat records bind to the same extent | Selected reaction records and checked binding |
| Reaction heat | Energy per reaction extent, J/mol | Heat source consumes the same signed/normalized extent convention | Authored heat record and existing directed-energy contribution |
| Pure/pair data | Coefficients retain their family's dimensions and physical contracts | Phase, subject orientation and variant are declared distinctions | Record/collection/form/model claims compose without losing their layer |
| Costing | Area, absolute pressure and currency basis remain typed | Empirical regions remain distinct from mathematical positivity and currency convention | Existing costing owners migrate through the generic claim/use gate |

**Well-posedness statement:** B1–B4 admit scientific inputs and applicability before execution; they do not replace declared variable roles, degree-of-freedom or structural analysis. This review does not establish those neighboring mechanisms or a solver's behavior. An admitted scientific selection alone never proves the assembled problem well-posed.

## 4. Representative changes and composition

| Scenario | Expected boundary and distinguishing control | Evidence |
|---|---|---|
| <a id="s01"></a>S01 — Add unknown and complete-empty species | Catalog admission preserves both; conserved reaction/translation/element projection claims demand complete evidence independently of masses | Proposed, B1 |
| <a id="s02"></a>S02 — Bind 2H₂ + O₂ → 2H₂O to material state | Missing water refuses; inert nitrogen contributes zero; one coefficient edit drives CSTR and PFR; mismatched kinetic/heat normalization refuses | Proposed, B2 |
| <a id="s03"></a>S03 — Compose a ternary from independent and joint fits | Existing A–B from X and A–C/B–C from Y can mix; same-publication variants remain distinct; orientation and declared fit closure govern admissibility | Proposed, B3 |
| <a id="s04"></a>S04 — Evaluate unknown/outside evidence under named permission | Unknown permission does not authorize outside evidence, extrapolation does not authorize unknown, and neither covers an unrelated record or bypasses a hard domain | Proposed, B4 |
| <a id="s05"></a>S05 — Lower a guarded derived argument in an active branch | Required observation and attribution survive simplification/differentiation even when numerical dependence cancels; inactive branches are not evaluated | Proposed, ADR-0141 |
| <a id="s06"></a>S06 — Change semantic selection or transport | Record/claim/convention/permission changes alter identity; generated boundary retains unknown/outside states and lineage; incompatible history is refused rather than silently reinterpreted | Proposed, ADR-0140/0141 and blueprint §20.5 |

Mechanism substitution is bounded here: replacing the numerical evaluator must preserve admitted demand/domain/observation semantics at the compiler/math boundary. The target does not expose evaluator-local handles as scientific identity. No alternate solver integration is proposed or qualified.

## 5. Mechanisms and numerical stages

| Stage / owner | Formulation and demand policy | Derivatives / scaling | Problem class, outcomes and checks |
|---|---|---|---|
| Modeling admission | Generic typed predicates/relations encode completeness, projection, selection closure and scoped claims | Not a derivative or scaling operation | Typed admission refusal; no solver starts |
| Specialization / compiler | Retain claim obligations by actual call/record/input; whole-interval coverage requires an explicit contract; hard domains are unconditional obligations | Preserve observation demands separately from numerical cancellation and differentiated values; existing physical/scaling owners remain | Generic evaluation yields Applicable/OutsideRegion/UnknownEvidence; permission gates use independently |
| Guarded math execution | Active branches evaluate their attributable obligations; inactive branches do not; numerical rewriting cannot erase them | Library-owned arithmetic/derivatives remain; this review makes no new derivative-order or accuracy claim | Domain failures remain failures under every permission; final convergence/closure and E3 qualification are outside this boundary |

Existing `specialize/envelopes.rs` lifts interval observations through unchanged arguments and refuses conditional or derived-argument cases; its intersection can also collapse attribution. It cannot establish S05 merely by renaming its observation type. The target requires replacing that limitation with demand-preserving generic evaluation. Existing guarded math regions supply the relevant separation from library arithmetic; they are not evidence that the new observations already execute there.

## 6. Foundations and gates

All judgments below concern the **Proposed target**, not conformance of today's implementation.

| Foundation | Judgment and reason |
|---|---|
| AP-01 | Satisfied: source science, selection coherence, use policy and generated representation have distinct owners |
| AP-02 | Satisfied: explicit complete/unknown, record shapes, normalization and observation states replace incidental defaults |
| AP-03 | Satisfied: independent fits compose; declared dependencies and alternative unions have different laws |
| AP-04 | Satisfied: the target models consequential distinctions and requires executed sources/evaluations to consume their admitted authorities |
| AP-05 | Satisfied: checks and typed refusals govern participant support, coherence, claim demand and permission scope |
| AP-06 | Satisfied: admission and policy operate on explicit package/binding inputs without a solver, database or complete workflow |

| Gate | Judgment at document-contract evidence |
|---|---|
| G1 Authority | Pass: source coefficients/composition own meaning; projections are immutable; permission does not rewrite evidence |
| G2 Semantic fidelity | Pass: absence, completeness, zero products, orientation, evidence basis and authorization remain distinct |
| G3 Validity | Pass: admission/evaluation boundaries and refusals are explicit, including hard domains under every permission |
| G4 Hidden behavior | Pass: named selection and permission are declared inputs, not ambient fallback or implicit selection |
| G5 Consistency/recovery | Pass for immutable admission products and retained observation/permission separation; durable publication/recovery excluded |
| G6 Transformation/reuse | Pass: pre-rewrite demands, conserved mappings and complete identity inputs are required; execution remains unverified |
| G7 Truthful claims | Pass: contracts are Proposed and qualification is expressly deferred; permission retains unknown/outside evidence |
| G8 Library leverage | Pass: no generic mathematical or solver replacement is proposed; reuse existing relational/predicate and guarded library boundaries |
| G9 Architectural fitness | Pass: all six target foundations are satisfied for S01–S06 |
| PS-G1 Physical consistency | Pass for the proposed conservation, extent, applicability and convention contracts; numerical closure untested |
| PS-G2 Well-posedness | Not applicable to this bounded admission-contract decision; structural/solver contracts are unchanged and not certified |
| PS-G3 Numerical integrity | Pass for proposed nonerasable hard-domain and observation obligations; derivatives, solver statuses and post-solve qualification are not certified |

## 7. Findings

No open target-contract MUST gap or actionable architectural defect was found in the final reviewed proposal. The plan/ADR permission ownership mismatch was corrected before this verdict. Source limitations above identify the implementation migration required by the accepted target, not completed corrections or implementation acceptance. No SHOULD exception is needed for the scoped target.

## 8. Library fit and ownership cost

The new scientific meaning belongs in authored domain declarations and their admitted products. Generic parsing, relational constraints, predicate representation, canonical pair handling, arithmetic, differentiation and typed transport remain with existing mechanisms/library integration owners. An external property library would not itself supply the selected records' completeness, fit identity, coherence or consuming authorization; adopting one would still need these same scientific contracts. No new library API, provider qualification or numerical algorithm is selected by this review.

The material cost is a coordinated migration of banks, selectors, methods, seed bindings and generated boundaries. That change is justified by new scientific distinctions, rather than a reusable procedural framework. Delete coefficient callbacks, package-wide pair-source selection, phase sentinels, observation-loss shortcuts and replaced tests/callers as their replacements are exercised.

## 9. Alternatives and tradeoffs

| Alternative | Judgment |
|---|---|
| Current sparse-zero formulas, callback sources and publication-wide selectors | Cannot distinguish unknown knowledge from conserved zero or give individual fits authoritative identity |
| Per-pair publication selector only | Improves composition but still conflates variants and coherent fit collections |
| Mandatory formulas/ranges or fabricated unbounded envelopes | Rejects legitimate incomplete catalog knowledge or upgrades unknown empirical evidence |
| Blanket analysis permission | Silently covers unrelated later selections and obscures authorization scope |
| Selected target / simplest viable design | One admitted scientific meaning over existing generic mechanisms; explicit record/family permissions and immutable projections meet S01–S06 without a universal ontology |
| Library-owned alternative | Existing library arithmetic and generic mechanisms coincide with the selected mechanism direction; they do not replace scientific evidence/authorization ownership |

Revisit if scientific extensions demand a second coefficient authority, inferred record interchangeability, sentinel key, or permission widened beyond its named target.

## 10. Verification and limits

| Claim | Evidence / settling check | Result |
|---|---|---|
| Contract ownership and distinctions | Proposed; independent reading of final ADRs/plan and inspected owners | Target accepted |
| Complete/empty/unknown and conserved projection | Proposed; B1 controls including elemental control-volume authority | Not run |
| Shared reactor source and extent conventions | Proposed; B2 participant/inert/both-reactor/normalization controls | Not run |
| Parameter identity and coherence | Proposed; B3 ternary, variants, asymmetric reversal, joint-fit projection/dependency and predictive-zero controls | Not run |
| Applicability and scope | Proposed; B4 independent permissions, unrelated-record refusal, union/dependency, whole interval, active branch/derived argument/cancellation and hard-domain controls | Not run |
| Identity and generated boundary | Proposed; changed claim/closure/convention/permission identities and observation round trips | Not run |
| Touched unit/property conformance and fidelity | Existing interfaces inspected; no executed campaign imported as evidence for new contracts | Unqualified implementation; Plan 25k owns integrated campaign |

No passing test count, numerical fidelity, performance or complete family migration is claimed. Correctness checks later use the explicit force-validation feature and recipe-owned targeted tests, against the zero failure target. A focused pass cannot establish whole-product or unexercised scientific qualification.

## 11. Authority changes and disposition

ADR-0140/0141 precede implementation and route the changed scientific identity, applicability and generated boundary contracts. ADR-0127 remains immutable: the new record amends its parameter/selection consequences while preserving chemical-core placement. Blueprint §9.1/§9.3/§9.7/§9.9/§9.10 and §14.3 take enduring meaning through the decision/design route, including the collection revision. Existing blueprint §20.5 remains the boundary against silent reinterpretation; this review authorizes no persisted-history migration.

[Plan 25's disposition table](../../plans/25-design-remediation.md#finding-dispositions) is the current finding owner; [Plan 25b](../../plans/25b-scientific-knowledge-and-applicability.md) owns B1–B4 implementation state and focused evidence. This review adds no independently maintained backlog. ADR acceptance does not resolve implementation findings or qualify behavior. E3 consumes observations without becoming a second applicability evaluator; G/25k retain their stated durable/integrated responsibilities.

## 12. Decision

**Accept the scoped proposed target in ADR-0140, ADR-0141 and Plan 25b B1–B4.** Architectural fitness and semantic contract adequacy are satisfied at **Proposed** evidence. The acceptance depends on the contracts as written: complete conserved evidence, immutable derived reaction and elemental projections, record-level identity/coherence, named nonwidening permissions, hard-domain refusal, retained demand/attribution and changed semantic boundary identity.

The implementation remains unaccepted and unqualified. A surviving independently editable coefficient callback, default-unrestricted missing claim, widened permission, collapsed observation, or unchanged identity for changed scientific meaning would falsify this target's guarantee and require revision.

Intended artifact: `docs/design_review/reviews/design_review_plan25b-contracts_2026-09-30.md`.
