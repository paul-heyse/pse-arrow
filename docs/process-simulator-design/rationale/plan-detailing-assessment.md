---
title: "Plan detailing and industry coverage assessment"
notion_source: "https://app.notion.com/p/40b141b58369811ba361d5fed2ea91e8"
notion_last_edited: "2026-11-01T23:33:59.136Z"
---

# Plan detailing and industry coverage assessment

Source: [Notion](https://app.notion.com/p/40b141b58369811ba361d5fed2ea91e8). Imported 2026-11-02.

**Status: Incorporated into the adopted design and implementation plan** · 2026-11-01. The user authorized actioning I01–I08; their contracts now appear in the owning domain pages, catalog and [Implementation design and IDAES equivalence plan](../delivery/index.md). This page preserves rationale and the incorporation map. The inspected repository baseline is `19e7b49a67df24c1bf73005a6186e145e930d53a`; P0–P7 remains the delivery sequence. Candidate industry cases remain separately scoped.

Adoption records: [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f) and [Flowsheet implementation and IDAES equivalence plan — v1](version-records.md#source-40b141b5836981ae9187c0bc93c10aeb).

## Assessment

The current plan has sufficient architectural direction to begin bounded implementation. The highest-value further detailing is to settle consequential semantic choices and observable behavior at the boundaries the coding agent will touch: source editing, identity, state/transport compatibility, scientific data, operating continuity and execution capability. Local code organization should remain delegated to the coding agent.

The broader ambition is feasible as a coherent process-modeling direction: industry libraries can share scientific authoring, typed physical relationships, composition, cases, preparation and evidence while expressing different materials and physical methods. Scientific adequacy remains specific to the selected model, data and intended engineering decision. No single case or method can qualify an industry.

The main architectural risk is allowing seed examples to define universal assumptions. The current molecular/molar liquid–vapor examples are useful implementations. Broader process modeling also needs characterized fractions, aggregate biochemical quantities, particles, polymer distributions, physical operating programs and pressure-driven networks. The adopted domain already provides most of the distinctions these require; exact package and numerical support needs concrete consumers. [Current model owner](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/models-and-composition.md); [current physical semantics](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/physical-semantics.md).

## Reading order

1. [Coding-agent implementation brief](../delivery/coding-agent-brief.md) — Fixed and delegated decisions, exact code seams, change invariants, extension decisions and four bounded next packets.
2. [Cross-industry domain contracts](../process/cross-industry-contracts.md) — Materials and representations, populations, conservation, inventories, operating continuity, capabilities, coupling and scientific evidence; ten representative process cases.
3. [Industry coverage and expansion strategy](../delivery/industry-coverage-and-expansion.md) — Nine industry groups, reusable capability families, specialist boundaries and candidate increments beyond the IDAES baseline.

## Adopted refinements and their owners

The stable references below identify the incorporated design and planning refinements. They are document references, not runtime identities, task statuses or acceptance authorities. Their detailed pages remain supporting specifications; the owning domain and implementation chapters now contain the operative guidance.

| Reference | Adopted refinement | When it matters |
| --- | --- | --- |
| I01 | Give each coding packet a bounded engineering outcome, owner, settled scientific choices, discriminating evidence and conditions that require a changed approach | Before executing the relevant packet; routine implementation choices remain delegated |
| I02 | State the editing representation, semantic identities, admitted scope and effects explicitly; classify numeric, structural and scientific edits | P1 authoring/inspection contracts |
| I03 | Define state/transport compatibility and substitutability for the actual requested outputs, conditions and derivatives | P1 public contracts; P2/P3 model substitutions |
| I04 | Preserve material identity, characterization, representation and conservation evidence as separate meanings | Before generalized material APIs; implement through the selected scientific families |
| I05 | Preserve physical inventory across operations; distinguish equipment, procedure, lot/campaign and computational study | P4 dynamics and P6 operating/campaign workflows |
| I06 | Distinguish exposed equations, input/output evaluation and stateful external time advance, with actual lifecycle capabilities | When a concrete external or procedural consumer enters scope |
| I07 | Carry scientific data, observations, assessment boundaries and result coverage through existing provenance/evidence owners | Each new package or engineering analysis |
| I08 | Use representative cross-industry cases to challenge contracts and expand a finite supported population | During relevant design choices and subsequent selected library increments |

### Incorporation map

| References | Live owners updated |
| --- | --- |
| I01–I02 | [Architecture and process authoring](../delivery/architecture-and-process-authoring.md); [Implementation sequence and first delivery](../delivery/implementation-sequence.md); coding-agent brief |
| I03–I04 | [Materials, states, and physical models](../process/materials-states-and-physical-models.md); [Scientific packages and equipment delivery](../delivery/scientific-packages-and-equipment.md); matching concept rows |
| I05 | [Cases, specifications, and studies](../process/cases-specifications-and-studies.md); procedure, lot and campaign contracts; P4/P6 roadmap guidance |
| I06 | Process structure and science chapters; [Cross-industry domain contracts](../process/cross-industry-contracts.md) |
| I07 | Observation/assessment concepts; science and [Equivalence and qualification](../delivery/equivalence-and-qualification.md) chapters |
| I08 | [Scope and modeling behavior](../process/scope-and-modeling-behavior.md); [Industry coverage and expansion strategy](../delivery/industry-coverage-and-expansion.md); IDAES scope and reference-register distinctions |

All entries above refer to documentation changes. They do not record executed implementation or qualification.

## Important code-level implications

The updated coding brief supplies concrete existing seams without prescribing a new class hierarchy:

- `with_declarations` produces an admitted declaration revision and explicitly does not retain old document text as the editing authority. Text editing and declaration editing must have an intentional representation policy.
- `StateKey` contains a contractual slot role and actual semantic indices. A presentation-label rename differs from changing the slot's scientific contract.
- `prepare_declared` handles Solve; `initialize_declared` preserves the complete initialization report needed by the failure-restoration journey. The convenience `execute_declared` route turns incomplete initialization into an error.
- `LocalStudy` deliberately has no serialization contract. Local execution and explicit recording retain their existing owners.

These details come from current source and affect the first implementation packets. [Source/edit APIs](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/modeling.rs); [declared procedures](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/modeling/declared.rs); [state identity](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-modeling/src/specialize/process.rs); [study lifecycle](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/study.rs).

## Scope horizons and sequencing

| Horizon | Role | Relationship to the existing plan |
| --- | --- | --- |
| Current implementation target | Pinned IDAES process capabilities, configurations, examples and native authoring workflows | Continue P0–P7 and the existing first recycle/exchanger delivery |
| Broad process architecture | Preserve the semantic distinctions required by characterized materials, populations, operating continuity, networks and external models | Clarify the contracts now where they affect chosen interfaces; use source/design examples to resolve actual uncertainty |
| Selected industry expansion | Deliver useful packages and qualified cases for the additional industrial requirements | Admit finite increments through existing planning/decision routes; do not silently enlarge the IDAES acceptance denominator |

These are product-scope horizons, not a second implementation roadmap. The broader ambition should influence current irreversible assumptions while remaining separable from work required to close the initial qualification target. A contract example may expose a gap or a truthful refusal without implementing a complete new industry flowsheet.

## Adopted next implementation packets

1. **Ordinary inputs:** establish admissible source/data/cases for the existing recycle and exchanger definitions, with an explicit editing representation.
2. **Inspection and edits:** expose selected unit/state/port/connection/case meaning; demonstrate a label rename, numeric input edit and topology edit through current owners.
3. **Complete native journeys:** exercise initialization/solve, the intended failure and restoration, design specifications, physical checks and semantic result inspection.
4. **Matched references:** add complete isolated IDAES correspondence while preserving independent expectations and existing qualification ownership.

The coding brief gives each packet a deliverable, evidence and a condition that requires resolving a scientific or architectural choice. These conditions do not create routine permission requests or a new review cadence. They identify shortcuts that would lose meaning or invalidate the result.

## Scientific information and operating context

Further detail should also make data and observation responsibilities explicit. Characterization, measurement methods, units/bases, uncertainty, applicable conditions and provenance belong with the selected inputs. NIST's ThermoML documents uncertainty, measurement method, constraints and bibliography at the data-point level, while distinguishing representation checks from critical evaluation. This is useful evidence for the required contract; adopting a ThermoML importer is a separate consumer-driven choice. [NIST ThermoML](https://www.nist.gov/mml/acmd/trc/thermoml).

For later operational use, map sensor/assay identifiers and timestamps into model observations through explicit adapters. Preserve acquisition time versus process time, quality flags, missing values and the uncertainty/error model. Actual equipment tags can be external references; a conceptual design model need not pretend to identify an installed asset. Parameter fitting, reconciliation and predictive validation should retain distinct data uses. These are adopted semantic requirements for relevant observation/result consumers. Implement them through the existing owners when selected; a historian or manufacturing system is not a prerequisite for the modeling core.

## Judgment on integration burden

Retain authored scientific packages and existing generic owners. Use an ordinary package/interface extension when it expresses the required science. Extend a generic contract only when a specific consumer cannot preserve its meaning through those constructs. A semantic unit boundary does not require a separate task, database query or native call; physical execution should suit the selected workload.

This assessment applies the repository's plan-improvement and functional-extension methods, Core 3.4, ProcessSimulator 1.5 and relevant test-assessment guidance. It introduces no new evidence registry, readiness framework, mandatory broad audit or universal plugin architecture. Architectural adequacy and scientific adequacy have separate conclusions: the existing architecture is a strong basis for extension, while broad industrial scientific coverage remains a substantial package/data/qualification program. [Review methods](https://github.com/paul-heyse/pse-arrow/tree/19e7b49a67df24c1bf73005a6186e145e930d53a/.codex/skills); [selected principles](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/standard.toml).

No repository code, numerical tests or qualification results changed during this documentation adoption. The detailed pages preserve alternatives, exact owner leads, representative cases and remaining uncertainty for implementation. The semantic contracts and coding guidance are now incorporated; future industry packages and scientifically unresolved inputs retain their explicit scope and evidence requirements.

[Coding-agent implementation brief](../delivery/coding-agent-brief.md)

[Cross-industry domain contracts](../process/cross-industry-contracts.md)

[Industry coverage and expansion strategy](../delivery/industry-coverage-and-expansion.md)
