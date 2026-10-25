---
title: Source-owned physical identities design review
date: 2026-10-25
tier: change
purpose: target
focus: principle-focused
standard: core-3.4/process-simulator-1.5
decision: Accept
evidence: Interface-checked
subject: docs/adr/0226-source-owned-physical-identities.md
disposition-owner: docs/plans/56a-model-meaning-and-physical-interpretation.md
doc_role: review
doc_owner: docs/plans/56a-model-meaning-and-physical-interpretation.md
doc_topics: [design, numerical-execution]
---

# Source-owned physical identities design review

## Assessment, scope and decision — slots 1 and 12

**Accept the proposed source-owned identity mechanism.** A declared physical identity should
reuse its existing `pse-quantity` Rust type. The registry supplies the domain and projections;
generated aliases connect it to the native owner. A second wrapper adds representation and
conversion boundaries without another scientific distinction.

This publishes the independent delegated design reviewer's judgment, with its repeated context
condensed by the coordinator. Acceptance concerns a **Proposed** target with **Interface-checked**
integration evidence. It does not establish implementation correctness, DA-005 closure or A03 completion.

| Field | Boundary |
|---|---|
| Subject | ADR-0226 source metadata, aliases and model/Arrow/Python/schema/PyO3 projections |
| Standard | Core 3.4, heuristics 1.0, Process Simulator 1.5, pse-arrow binding |
| Tier / purpose / focus | Change / target / principle-focused; AP-01/02/04, DP-01/04/14, PS-01 with relevant full-standard guarantees |
| Reviewer | Independent delegated Codex design reviewer, 2026-10-25 |
| Source | `6d83bfc434e5ef57e9dae2438e56f75cf18e8ba6`, proposed ADR and Plan56/56a decision text |
| Audit | Publication `f9b96810a`, audited source `682fc3830f8a370f0995544af5adf821e4be9ab9`, DA-005 and adjacent DA-003/006 |
| Owner | Plan56a A03; Plan56 owns finding/rule dispositions |
| Excluded | Full reader, A01/A02 correctness, solver/formulation/performance, store migration, RC01–RC06 |

Relevant identity interfaces were unchanged between the audit source and committed baseline.
Concurrent A01/A02 edits are excluded. Constructor/generation preparation began during review
closeout; the judgment assesses the proposed target and inspected interfaces, not emerging code.
DA-005's corrected scope remains: physical admission works and bundles retain authored physical
meaning; direct typed/readable public interpretation is incomplete. This mechanism is a prerequisite.

## Meaning, authority and integration — slots 2, 3 and 5

`pse-quantity` owns physical types and membership/conversion admission. `pse-schema` owns declared
domains, carrier/base/FK/owner resolution. `pse-codegen` owns mechanical projections. A source path
identifies an implementation; it does not change physical identity, framing or persisted meaning.

Quantity references retain complete quantity contracts, canonical units, basis, datum, point/
difference, shape and subject. Unit identities retain dimension, affine/datum restrictions and
registered conversion policy. Equal storage bases do not merge different domains; equal dimensions
or symbols do not establish compatible physical contracts. An alias supplies no physical admission.

The existing emitter also produces SemanticEq/HeapUsage/SemanticFrame, Arrow codecs and the
semantic-identity inventory used by PyO3. Alias emission must retain these. Their traits are owned
by receiving crates, providing an inspected implementation route for the existing physical type.

Retain declared carriers, compatible bases, inherited FK conflicts, explicit single-key ownership
and unused-declaration rejection. Preserve source metadata through generated IdentitySpec
reconstruction; otherwise regenerated registries could silently return to wrapper emission.
Source relationships distinguish implementation attribution from scientific identity. Assembly
validates declaration/base constraints; compilation and focused contracts validate that the named
native type supplies the required operations. A path string alone cannot establish its base.

### Consequential schema seam

Physical IDs currently use the default typed-ID macro arm, whose schema delegates to SemanticId.
The registry arm adds `x-pse-identity`; the document emitter consumes it to choose a domain-specific
Python alias. Rust aliasing alone would leave the public documents insufficiently typed. ADR-0226
explicitly requires that facet at the physical schema owner.

Python NewTypes remain annotations. Preserve strict native value-class/width admission and let
the physical owner reject invalid membership and combinations. No formulation, evaluation, solver,
degrees-of-freedom or derivative/scaling mechanism is introduced or qualified.

## Scenarios and alternatives — slots 4, 8 and 9

| Scenario | Required distinction |
|---|---|
| S01 declare existing physical domain | One native type, domain-specific Python/schema projection |
| S02 swap quantity/unit identities | Typed Rust substitution refused; physical admission validates public combinations |
| S03 move native module | Generation metadata changes; identity bytes and frames do not |
| S04 model/Arrow/PyO3 roundtrip | Exact bytes, null behavior, schema facets and role inventory survive |
| S05 change current context after result | Original retained closure or explicit unavailable; alias alone does not deliver this reader |

| Alternative | Judgment |
|---|---|
| Bare SemanticId projections | Small immediate work but domain interchangeability persists |
| Independent generated physical wrappers | Second native representation and repeated conversion seams |
| Move physical types into generated model owner | Unnecessary reversed dependency without scoped benefit |
| Handwritten independent aliases/types/role lists | Reuses types but creates parallel maintained inventories |
| Optional source metadata | Selected: existing owner and projections, ordinary generation for unsourced identities |

No new external library is needed. Existing aliases, typed-ID machinery and schema/Arrow/PyO3
adapters fill the responsibility. Source-based assessment required no library API research.
Qualitatively the route removes conversion obligations and adds publication-time metadata without
per-value lookup/copy, numerical crossing, transaction or cache. No speed/capacity claim is made.

## Architectural assessment and gates — slot 6

| Foundation | Verdict and basis |
|---|---|
| AP-01 | Satisfied: physical admission/meaning and mechanical projection retain owners |
| AP-02 | Satisfied: consumed native contract reused, incompatible base changes explicit |
| AP-03 | Satisfied: existing mechanisms compose without parallel workflow |
| AP-04 | Satisfied: distinct domains refer to complete physical contracts and existing owner |
| AP-05 | Satisfied: source/carrier/base/ownership explicit |
| AP-06 | Satisfied: pure generator tests need no solver/store; actual bridges still tested |
| AP-07 | Satisfied: publication metadata avoids another runtime representation |

G1–G3 pass at target-design level: one authority, semantic fidelity and preserved validity
controls. G4 passes: no implicit runtime effect/conversion. G5 passes within this change: no new
recovery protocol and coherent regeneration remains required. G6–G8 pass at target-design level:
unchanged transformations/frames, truthful interface evidence and reused capabilities. G9 passes
independently on applicable foundations. PS-G1 passes for the proposed identity mechanism;
PS-G2/PS-G3 are not applicable because formulation/numerical execution are unchanged.

## Findings and verification — slots 7 and 10

No new blocking design finding is established. The existing base-only schema facet is a known
implementation seam explicitly addressed by the target, not an omitted design requirement.
Acceptance includes carriers/bases/FK/ownership, one native type, all traits/codecs/role inventory/
provenance, domain schema facets/Python aliases, exact bytes/serde/frames and physical-owner
membership/conversion. Historical meaning uses original retained context or explicit unavailability.

**Interface-checked:** declaration/assembly, physical macro/schema, identity/sourced-enum emitters,
registry reconstruction, Python aliases, document annotations, PyO3 roles and blueprint §4.1,
§5.1, §8.1, §21.1. **Not run by reviewer:** compilation, generation, tests, native/Python journeys,
benchmarks or physical reference comparisons. The owning plan must supply independent alias/base/
schema/byte controls and actual boundary exercises; regeneration alone cannot prove correctness.

## Rule impact — slot 11

<a id="rc07"></a>**RC07: permit source-owned identity aliases.** ADR-0115 Outcome 2 and blueprint
§5.1 currently describe generating wrappers for every declared identity. ADR-0226 permits an
existing owner and generated aliases/projections. The operator explicitly selected this narrow
route. Keeping the old rule would require a different single-owner arrangement; duplicate wrappers
do not preserve the selected design. Plan56/P00 owns that disposition and A03 its execution.
RC01–RC06 remain unadopted. Formal ADR status is separate from implementation authorization.

## Decision — slot 12

Behavioral/semantic adequacy is sufficient as a Proposed bounded target. Architectural fitness
is satisfied at the inspected interface/design level. **Overall: Accept**, including all stated
schema/carrier/trait/codec/provenance/framing obligations. Product acceptance and complete original-
context readability remain with Plan56a.
