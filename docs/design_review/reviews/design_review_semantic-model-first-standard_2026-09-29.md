# Review: semantic-model-first standard adoption

## 1. Scope, drivers and coverage

**Author review, 2026-09-29; design/target.** Subject: core/template 3.2, process-simulator
guidance 1.2, binding, agent roles and process skills, AGENTS and blueprint §24.4 under
ADR-0128. This assesses the maintainer-requested policy adoption, not product architecture
or conformance. The implementation is the changed guidance; effectiveness in later product
changes remains Proposed. Method: source inspection and the three scenarios below; no
independent reviewer or behavioral pilot is claimed.

## 2. Decomposition, ownership and dependencies

The core owns the domain-model MUST and AP-04; the template makes it decisive through G9.
The profile supplies scientific interpretation and retains its own gates. The binding routes
local owners; agent guidance applies those contracts. Library exploration and library capability
skills are outside scope. Generated Codex roles derive from the canonical Claude role.

## 3. Contracts, authority and constraints

Core §1 requires an explicit, adequate model of supported phenomena and operations. AP-04
requires behavior to realize its scoped authorities. DP-02/03/05/06/08 distinguish domain
meaning, invariant ownership, contextual bindings, reuse and operation/translation contracts.
Ordinary functions qualify; a DSL, registry or universal schema is not a requirement. A
MUST gap cannot be deferred into acceptance for the same supported behavior.

## 4. Change scenarios and composition

| Scenario | Expected boundary and inspected route | Judgment |
|---|---|---|
| <a id="s01"></a>S01: standardize output rows while separate workflows define their meaning | Core §1/AP-04, template slots 2–4/6 and the core skill require tracing phenomena and operations into consumers; G9 cannot accept the missing authority merely because outputs match | Satisfied at specification strength |
| <a id="s02"></a>S02: implement a domain operation as an ordinary typed function | Core §1/DP-08 and template slot 3 accept an owned function contract; no registry, instruction object or extra crate is required | Satisfied at specification strength |
| <a id="s03"></a>S03: centralize a definition that still collapses a consequential distinction | AP-04 independently asks whether the model is adequate; one owner cannot compensate for a missing distinction; scientific gates remain independent | Satisfied at specification strength |

Classifying instances, bindings, compositions, policies, concepts and mechanisms makes change
ownership inspectable without promising a fixed edit count or cheap incompatible substitution.

## 5. Mechanisms and execution

Documentation policy and generated agent-role synchronization only. No runtime operation,
solver, schema migration or library integration changes. Historical standard versions remain
attached to historical reviews; the manifest selects the new edition for subsequent reviews.

## 6. Architectural assessment and gates

| Foundation | Verdict and evidence |
|---|---|
| AP-01 | Satisfied: core, profile, binding and agent responsibilities remain distinct |
| AP-02 | Satisfied: IDs and role contracts persist; historical verdicts retain their edition |
| AP-03 | Satisfied: existing slots and instructions compose; no parallel assessment framework |
| AP-04 | Satisfied: one governing requirement and consistent adequacy/authority assessment, S01–S03 |
| AP-05 | Satisfied: MUST gaps and G9 consequences are explicit |
| AP-06 | Satisfied: small changes keep their bounded scenario; ordinary functions suffice |

| Gate | Verdict and scope |
|---|---|
| G1 | pass: the core owns meaning; other layers apply it |
| G2 | pass: domain distinctions and the existing physical obligations remain explicit |
| G3 | not applicable: no runtime admission or validator changes |
| G4 | not applicable: no execution effects changed |
| G5 | not applicable: no publication/recovery protocol changed |
| G6 | pass: specification translations into profile and role guidance preserve the requirement |
| G7 | pass: documented policy is distinct from product qualification or measured improvement |
| G8 | not applicable: no library exploration or implementation changes |
| G9 | pass for this policy's source design, on the foundation judgments above |
| PS-G1–PS-G3 | not applicable: no scientific implementation changed; requirements retained |

## 7. Findings

No material gap in the bounded policy adoption. This is an author assessment, not independent
evaluation of review effectiveness, and makes no claim about the current product's alignment.

## 8. Library fit and ownership cost

Unchanged, outside this adoption. Full library eligibility and existing exploration remain.

## 9. Alternatives and tradeoffs

A governing paragraph alone would leave review instructions incomplete. A new foundation,
gate or prescribed modeling framework would duplicate existing owners or add machinery.
Strengthening AP-04/G9 and existing slots keeps the remaining foundations independent.

## 10. Verification

S01–S03 are reasoned policy traces, not executed agent trials. Document checks validate links
and generated role synchronization only; product tests and numerical qualification are not_run
for this scope. Future ordinary reviews provide evidence of effectiveness.

## 11. Authority changes and disposition

ADR-0128 owns the adoption rationale; blueprint §24.4 owns the local policy, with revision 82.
No product finding, active plan or deferred scientific decision is closed by this review.
The maintainer explicitly authorized equivalent adoption of the already reviewed wording.

## 12. Decision

**Accept** for the implemented documentation policy: architectural fitness and semantic
adequacy of the guidance hold at inspected specification strength. Product conformance and
measured extension benefits remain unassessed. Apply the edition at subsequent scheduled reviews.
