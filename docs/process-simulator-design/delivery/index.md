---
title: "Implementation design and IDAES equivalence plan"
notion_source: "https://app.notion.com/p/40b141b5836981b6a2b3f3e52d80ad13"
notion_last_edited: "2026-11-02T05:11:15.667Z"
---

# Implementation design and IDAES equivalence plan

Source: [Notion](https://app.notion.com/p/40b141b5836981b6a2b3f3e52d80ad13). Imported 2026-11-02.

**Status: Adopted high-level implementation plan** · 2026-11-01. This plan applies the adopted [Flowsheet Process Domain Model](../process/index.md) to the current repository. It defines a delivery program and its acceptance criteria; it does not declare implementation or scientific qualification complete.

## Recommendation

Complete and expose the existing authored process-modeling system, then expand its scientific packages through complete engineering cases. The repository already owns source admission, state and port semantics, composition, mathematical preparation, native analyses and retained results. The new process domain should make those responsibilities coherent for a model author and guide the missing scientific content.

The primary effort is the breadth of scientifically correct property, reaction, equipment and application models, followed by their qualification. The architecture should let that work remain local to its scientific owner. New generic contracts are warranted when a real case exposes a missing representational or execution capability. [Current architecture](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/architecture-overview.md); [process specialization](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-modeling/src/specialize/process.rs).

## Detailed chapters

1. [Architecture and process authoring](architecture-and-process-authoring.md) — Ownership, process views, authoring/editing, preparation, execution, and principle traceability.
2. [Scientific packages and equipment delivery](scientific-packages-and-equipment.md) — Thermodynamic, material-data, state, reaction, equipment, inventory, and provider contracts.
3. [IDAES scope and capability coverage](idaes-scope-and-capability-coverage.md) — Released capability families, non-notebook sources, configuration coverage, and scope boundaries.
4. [Equivalence and qualification](equivalence-and-qualification.md) — Matched physics, independent checks, comparison rules, truthful outcomes, and qualification ownership.
5. [Implementation sequence and first delivery](implementation-sequence.md) — P0–P7 dependencies, concrete completion criteria, and the first three existing-case witnesses.
6. [Reference case register](../reference/notebook-families.md) — 55 documented notebook families, plus held/archive and developer dispositions with pinned source links.

## Adopted implementation refinements and industry ambition

[Plan detailing and industry coverage assessment](../rationale/plan-detailing-assessment.md) — Incorporated on 2026-11-01. I01–I08 now inform the owning architecture, scientific, qualification and roadmap chapters and process-domain v1.1. The coding brief supplies precise source/edit/lifecycle guidance; the industry strategy supplies adopted semantic direction and candidate later increments. P0–P7 and the finite IDAES qualification target remain the delivery baseline.

Revision records: [Flowsheet Process Domain Model — v1.1](../rationale/version-records.md#source-40b141b5836981958b9ad5a14519520f) and [Flowsheet implementation and IDAES equivalence plan — v1](../rationale/version-records.md#source-40b141b5836981ae9187c0bc93c10aeb).

Start with [Coding-agent implementation brief](coding-agent-brief.md) when translating this high-level plan into the next bounded repository packets.

## Proposed scientific sourcing detail

[Scientific sourcing and catalog architecture — review and proposed detail](scientific-sourcing-and-catalog.md) — Review dated 2026-11-01. Records the user's direction for targeted scientific sourcing within pse-arrow's Rust codebase and existing SurrealDB instance. Details native typed imports, canonical publication, scientific discovery and reusable unit composition through physical structures, constitutive closures and engineering templates. The earlier thermo-knowledge dependency is superseded. Delivery mechanics remain proposed; the P0–P7 baseline is retained.

## Reviewed baselines

| Source | Exact baseline | Role |
| --- | --- | --- |
| [pse-arrow](https://github.com/paul-heyse/pse-arrow/tree/19e7b49a67df24c1bf73005a6186e145e930d53a) | `19e7b49a67df24c1bf73005a6186e145e930d53a` | Current implementation, supplied architecture documents and design principles |
| [IDAES-PSE 2.13.0](https://github.com/IDAES/idaes-pse/tree/84532d7f99a9f976b05170948147afdd69cfa5f6) | `84532d7f99a9f976b05170948147afdd69cfa5f6` | Released scientific and functional target |
| [Modern IDAES examples](https://github.com/IDAES/examples/tree/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675) | `eb64e38ac429b7e3ae4f8db071f8d8cad2eef675` | Published workflows, their local modules and data |
| [IDAES compatibility](https://github.com/IDAES/idaes-compatibility/tree/8f1ae7a3ca358625128a78722bc2fdb75753a441) | `8f1ae7a3ca358625128a78722bc2fdb75753a441` | Additional behavior, physical-verification and robustness references |

The examples snapshot declares package version 2.10.0 and an unpinned optional IDAES dependency. Its compatibility with core 2.13 must be established case by case in the frozen reference environment. The source review identified 55 documented notebook families; generated variants, held/archive cases, core examples and configuration coverage remain distinct. The separate case-register chapter records exactly what was inspected. [Examples package](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/pyproject.toml); [documentation contents](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/_toc.yml).

## Starting point and material gaps

| Area | Starting point | Main implementation obligation |
| --- | --- | --- |
| Process authoring | Authored definitions, immutable source revisions, state/port/inventory semantics, cases and native operations | A coherent process inspection/edit/compose/specify/execute journey over those identities |
| Thermodynamics and reactions | Ideal/PR, SRK records, NRTL, nonassociating PC-SAFT, selected states/equilibria/caloric methods and reaction projection | Required state/property configurations, data, steam/CO2, transport, electrolyte/speciation and reaction breadth |
| Equipment and dynamics | A substantial seed library; native ODE/index-1 and advanced analysis routes | Full generic/specialized configurations, multiphase storage and distributed temporal accumulation |
| Equivalence evidence | Authored conformance, four live parity exchanges and broader retained campaign owners | Complete matched cases, independent physical checks and actual scope-end qualification |

Source availability and historical passing evidence do not establish complete current qualification. Current limitations, including spatial-volume temporal dynamics and stored-transport projections, are explicit in source. [Current scope](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/scope-and-open-design.md); [current parity routes](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/justfile).

## Implementation commitments

1. Scientific definitions, parameter data and equations remain authored once in existing packages. Process views reference existing declaration, instance and semantic-member identities.
2. Equipment composes ports, states, closures, optional balance regions and subordinate models. New equipment is ordinarily package work, with supported configurations made explicit.
3. Cases bind operating/design conditions, observations, decisions and requested answers to the same model. Initialization and numerical policy retain their separate meanings.
4. Topology, equation incidence and execution decomposition remain separate graphs. UI layout is presentation; process edits use the existing admission boundary.
5. Generic gaps extend their owning language/schema/compiler/runtime contract once. Product/durable projections use the existing schema authority; symbolic and numerical work remains with current libraries.
6. Existing conformance, attempt, completion and publication owners retain evidence. New cases extend those mechanisms and preserve Plan 53/58h responsibilities.
7. Public contracts retain material characterization, component/balance bases, population structure and inventory meaning without forcing all materials into molecular/molar liquid–vapor seeds. Exact chemistry retains its stronger guarantees.
8. Physical procedures and campaigns preserve state carryover and transfers independently of computational Studies. External equation exposure, evaluation and time advance declare actual capability and effect boundaries.
9. Scientific inputs, observation mappings and engineering assessment boundaries remain attributable. Compatible interfaces, successful solves and matched arithmetic do not establish unqualified scientific equivalence.

These commitments apply the repository's Core 3.4 and ProcessSimulator 1.5 principles, particularly one fact owner, explicit physical meaning, composition, extension locality, complete dependency-based reuse and truthful scientific outcomes. The architecture chapter maps principles to concrete acceptance questions. [Selected standard](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/standard.toml).

## Delivery program

| Phase | Outcome |
| --- | --- |
| P0 | Freeze the complete target population and reconcile the owning scope documentation |
| P1 | Deliver the process authoring journey and first matched coupled cases |
| P2 | Complete the property/state/reaction dependencies needed by the portfolio |
| P3 | Complete generic equipment configurations and steady integrated workflows |
| P4 | Add missing inventories, distributed temporal physics and qualified transient cases |
| P5 | Deliver power, capture/membrane, gas-solid, adsorption/electrochemistry and gas-network families |
| P6 | Complete analysis, control, grid/multiperiod, surrogate and native inspection/reporting workflows |
| P7 | Close required case/configuration coverage and substantiate the scoped release claim |

The sequence is dependency-driven. Independent scientific families can proceed concurrently after shared contracts stabilize. Dates and staffing should be estimated after P0 resolves the case/configuration population and P1 establishes a complete delivery path.

## First concrete delivery

The [Implementation sequence and first delivery](implementation-sequence.md) incorporates packets A–D: ordinary admissible inputs; selected inspection and edits; complete native journeys; matched independent references and P1 closeout. The [Coding-agent implementation brief](coding-agent-brief.md) fixes their observable behavior and evidence while delegating local implementation organization.

Use the existing recycle-flash success and initialization-failure fixtures, plus the solved-area exchanger/costing fixture. Expose the normal authoring-to-result journey, preserve their independent checks and add isolated matched IDAES counterparts. Provide admissible ordinary inputs using the same definitions; fixture-only data must retain its conformance permissions. The exchanger's explicit applicability allowances establish scoped arithmetic, not a qualified purchase price. [Recycle cases](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/campaign/models/recycle-flash.pse); [exchanger case](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/campaign/models/exchanger-costing.pse).

## Meaning of equivalence

Reference reproduction aligns physical assumptions, parameterization, conditions and requested outputs. Deliberate scientific or numerical improvements receive separate evidence. Acceptance includes representability, original physical correctness, result correspondence, numerical behavior and workflow completeness, including expected diagnostics for deliberately invalid cases.

Every required case/configuration must have accepted evidence before the corresponding full claim. The 12 earlier domain witnesses establish useful architecture probes; the 55-family notebook register establishes a research inventory. Neither alone establishes equivalence. Whole-distribution equivalence additionally requires supplemental applications such as MatOpt, kept outside the flowsheet ontology. The plan's primary program covers the released process-modeling scope and identifies those supplements explicitly.

No repository implementation, builds, solver runs or parity campaigns were performed for this documentation update. The high-level plan and refinements are adopted; implementation progress and scientific qualification remain to be established through the repository's existing change and evidence owners. Historical baselines remain recorded in their revision records.

[Architecture and process authoring](architecture-and-process-authoring.md)

[Scientific packages and equipment delivery](scientific-packages-and-equipment.md)

[IDAES scope and capability coverage](idaes-scope-and-capability-coverage.md)

[Equivalence and qualification](equivalence-and-qualification.md)

[Implementation sequence and first delivery](implementation-sequence.md)

[Reference case register](../reference/notebook-families.md)

[Plan detailing and industry coverage assessment](../rationale/plan-detailing-assessment.md)

[Scientific sourcing and catalog architecture — review and proposed detail](scientific-sourcing-and-catalog.md)
