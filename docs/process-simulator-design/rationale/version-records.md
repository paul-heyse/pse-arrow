---
title: "Version Records"
---

# Version Records

<a id="source-40a141b58369812b9f43eb8609631238"></a>

## A Unified Domain Model for Chemical Simulation — v1 (31 Oct 2026, decisions confirmed)

Source: [Notion](https://app.notion.com/p/40a141b58369812b9f43eb8609631238).

**Summary:** One SimulationProblem = Model + Specification + Query (+ advisory MethodRequest) as the single interface between formulating and executing any chemical simulation task. Acausal models, roles in the Query, five closed answer kinds, Study as a layer above.

**Version:** v1 (31 Oct 2026, decisions confirmed)

**Decisions:** [The mesh is execution, not part of the problem](design-decisions.md#source-40a141b58369814288c3e94bdb793ff8); [Infer is a first-class answer kind](design-decisions.md#source-40a141b5836981a98488ca181dbcd28e); [Symbolic relations by default; opaque only where knowledge exists only as code](design-decisions.md#source-40a141b5836981b3a336c583de758998); [Structured decision spaces are in scope as a value type; solvers later](design-decisions.md#source-40a141b5836981c9a389f2cab9cd4224); [SimulationProblem stays atomic; the Study is a layer above](design-decisions.md#source-40a141b58369815faef9dc7af6fa206d); [One Expect covers input uncertainty and intrinsic stochasticity](design-decisions.md#source-40a141b58369817a844cdefc929ea71a); [Execution is a set of translation kernels, not a single router](design-decisions.md#source-40a141b5836981ad9bf4dbf3e8bbbd78); [Next pass makes every conceptual component and its hierarchy explicit](design-decisions.md#source-40a141b583698193907ed1fa9590cc03)

**Status:** Superseded

**Area:** Domain model; Execution

**date:Last synced:start:** 2026-10-31

**date:Last synced:is_datetime:** 0

**Working copy:** https://claude.ai/artifact/J5oKwfVpUTfcyEFYAnEEZJ

**Next version:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b583698131bd16c51b551f1d9f)

**Name:** A Unified Domain Model for Chemical Simulation

The superseded full draft remains at the source link above. Current design is imported under [simulation](../simulation/index.md); all version properties are retained here.

<a id="source-40a141b583698131bd16c51b551f1d9f"></a>

## A Unified Domain Model for Chemical Simulation — v2 (31 Oct 2026, deeper revision)

Source: [Notion](https://app.notion.com/p/40a141b583698131bd16c51b551f1d9f).

**Summary:** Every concept catalogued once with owner, references and rules; ownership tree and reference graph; contracts for the five answer kinds; execution as a population of kernels; six worked examples. Adds Regime to the Query and moves all bounds out of the Specification.

**Version:** v2 (31 Oct 2026, deeper revision)

**Decisions:** [Versioned definitions are referenced by model instances](design-decisions.md#source-40a141b58369814084d2df8fc1e8654c); [Bounds retain their provenance; the Query selects active enforcement](design-decisions.md#source-40a141b58369813eadace338d5d21224); [Regime is analysis intent constrained by model capabilities](design-decisions.md#source-40a141b583698171ad14c4860b2f2d0e)

**Status:** Superseded

**Area:** Domain model; Execution

**Previous version:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b58369812b9f43eb8609631238)

**date:Last synced:start:** 2026-10-31

**date:Last synced:is_datetime:** 0

**Working copy:** https://claude.ai/artifact/J5oKwfVpUTfcyEFYAnEEZJ

**Next version:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728)

**Name:** A Unified Domain Model for Chemical Simulation

The superseded full draft remains at the source link above. Current design is imported under [simulation](../simulation/index.md); all version properties are retained here.

<a id="source-40b141b583698185aee7fac52f7213f2"></a>

## Flowsheet Process Domain Model — v1 — v1 (1 Nov 2026)

Source: [Notion](https://app.notion.com/p/40b141b583698185aee7fac52f7213f2).

**Summary:** Defines the flowsheet product domain, 45 process contracts, explicit mapping to the overall framework, 12 acceptance workflows and repository evidence boundaries.

**Version:** v1 (1 Nov 2026)

**Decisions:** [Bounds retain their provenance; the Query selects active enforcement](design-decisions.md#source-40a141b58369813eadace338d5d21224); [Versioned definitions are referenced by model instances](design-decisions.md#source-40a141b58369814084d2df8fc1e8654c); [Regime is analysis intent constrained by model capabilities](design-decisions.md#source-40a141b583698171ad14c4860b2f2d0e); [Flowsheet process modeling is the product domain; the overall model is its reusable framework](design-decisions.md#source-40b141b5836981a48565fc5bb95aa7f8); [Authored definitions remain the single executable model authority](design-decisions.md#source-40b141b5836981fda780eb758b00d078); [Units compose zero or more balance regions and submodels](design-decisions.md#source-40b141b5836981d28c9fe407531c5f38); [Material states, ports, streams and connections have distinct contracts](design-decisions.md#source-40b141b5836981a396e3c0bebf08f087); [Physical choices, equivalent formulations and numerical approximations remain distinct](design-decisions.md#source-40b141b58369811cba09ddd746667e97); [Answer kinds are extensible task profiles with composable requirements](design-decisions.md#source-40b141b5836981e2a8fcf88ce59fbe96); [Validation and scientific acceptance report qualified evidence](design-decisions.md#source-40b141b58369816cb748c04bfe69e5dc); [Stable semantic identity and qualified reconstruction preserve traceability](design-decisions.md#source-40b141b583698138a397edbd352c8966); [Process cases and operating sequences reuse the overall task lifecycle](design-decisions.md#source-40b141b583698197a991f73e23cf49e9); [Representative IDAES workflows establish domain and implementation coverage](design-decisions.md#source-40b141b5836981ed8963d64a2bc8062f)

**Status:** Superseded

**Area:** Domain model; Architecture; Repository mapping

**date:Last synced:start:** 2026-11-01

**date:Last synced:is_datetime:** 0

**Working copy:** https://app.notion.com/p/40b141b5836981568752c1b6a854ddf0?pvs=204

**Next version:** [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Name:** Flowsheet Process Domain Model — v1

**Historical revision record.** Superseded by [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f) on 2026-11-01. The original v1 adoption rationale and source baseline below are preserved. Linked live pages and the concept catalog now carry the v1.1 refinements.

> Version 1 of the process-specific domain design. The linked hub, five chapters and structured catalogs are the live design authority. This record preserves the adoption basis, scope and evidence boundary.

#### Adoption and scope

Paul requested this companion design while authorizing the overall model revision on 2026-11-01. Flowsheet process modeling is the product focus: equipment composition, material and energy transfer, property and reaction behavior, operating cases and engineering studies. Detailed local equipment physics is admitted through explicit contracts; a universal CFD, molecular or electronic-structure framework is outside the immediate product scope.

#### Published design

[Flowsheet Process Domain Model](../process/index.md)

1. [Scope and modeling behavior](../process/scope-and-modeling-behavior.md)
2. [Process structure and connectivity](../process/structure-and-connectivity.md)
3. [Materials, states, and physical models](../process/materials-states-and-physical-models.md)
4. [Cases, specifications, and studies](../process/cases-specifications-and-studies.md)
5. [Repository mapping and acceptance workflows](../process/repository-mapping-and-acceptance.md)

The design includes 45 process concepts across structure, materials, physics, connectivity, operations, analysis and evidence; a 12-case IDAES workflow coverage matrix; and a worked exchanger example spanning rating, sizing, optimization, shared-parameter fitting and dynamics.

#### Central decisions

- Reuse the repository's authored definitions and semantic identities as the executable authority.
- Compose units from zero or more balance domains, states, constitutive closures and child models.
- Separate local material states, transported observations and stored inventories.
- Treat ports as typed boundary views and streams as views of identified connections or boundaries; physical link behavior has explicit equations.
- Keep physical stages, geometry, operating modes and events distinct from meshes, initialization and Study orchestration.
- Map process cases and atomic tasks to the existing Specification / Query / SimulationProblem contracts.
- Require attributable physical checks, qualified numerical evidence and explicit implementation limits.

#### Evidence baseline

Source review is pinned to [pse-arrow 9266555](https://github.com/paul-heyse/pse-arrow/tree/926655517f6eac4ab52ae7a817a7b72168afdc25). Its [README](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/README.md) pins selected comparisons to IDAES 2.13.0. Primary IDAES and related references are linked in the chapters.

No new simulator or parity run was performed during this design update. “Partial source evidence” and “Design case” are design-review statuses. They do not establish complete implementation or scientific parity.

#### Companion version

[Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728)

This is a version record and index to the live structured design, not a frozen copy of every page or database row. Future consequential changes should retain their rationale and evidence in the Decisions log and a successor version record.


<a id="source-40b141b5836981958b9ad5a14519520f"></a>

## Flowsheet Process Domain Model — v1.1 — v1.1 (1 Nov 2026, implementation and industry contracts)

Source: [Notion](https://app.notion.com/p/40b141b5836981958b9ad5a14519520f).

**Summary:** Incorporates I01–I08: precise process authoring and capability contracts, broader material and operating semantics, scientific evidence boundaries, and a 51-concept catalog. Retains the overall v3 framework and finite IDAES delivery scope.

**Version:** v1.1 (1 Nov 2026, implementation and industry contracts)

**Decisions:** [Bounds retain their provenance; the Query selects active enforcement](design-decisions.md#source-40a141b58369813eadace338d5d21224); [Versioned definitions are referenced by model instances](design-decisions.md#source-40a141b58369814084d2df8fc1e8654c); [Regime is analysis intent constrained by model capabilities](design-decisions.md#source-40a141b583698171ad14c4860b2f2d0e); [Flowsheet process modeling is the product domain; the overall model is its reusable framework](design-decisions.md#source-40b141b5836981a48565fc5bb95aa7f8); [Authored definitions remain the single executable model authority](design-decisions.md#source-40b141b5836981fda780eb758b00d078); [Units compose zero or more balance regions and submodels](design-decisions.md#source-40b141b5836981d28c9fe407531c5f38); [Material states, ports, streams and connections have distinct contracts](design-decisions.md#source-40b141b5836981a396e3c0bebf08f087); [Physical choices, equivalent formulations and numerical approximations remain distinct](design-decisions.md#source-40b141b58369811cba09ddd746667e97); [Answer kinds are extensible task profiles with composable requirements](design-decisions.md#source-40b141b5836981e2a8fcf88ce59fbe96); [Validation and scientific acceptance report qualified evidence](design-decisions.md#source-40b141b58369816cb748c04bfe69e5dc); [Stable semantic identity and qualified reconstruction preserve traceability](design-decisions.md#source-40b141b583698138a397edbd352c8966); [Process cases and operating sequences reuse the overall task lifecycle](design-decisions.md#source-40b141b583698197a991f73e23cf49e9); [Representative IDAES workflows establish domain and implementation coverage](design-decisions.md#source-40b141b5836981ed8963d64a2bc8062f); [Cross-industry material models preserve characterization, representation and scientific evidence](design-decisions.md#source-40b141b58369810d9cdec738fceffbca); [Physical operating continuity and external execution capabilities remain explicit](design-decisions.md#source-40b141b58369814f9a9df88bd2f862d3); [Adopt bounded coding packets with finite IDAES qualification and separately scoped industry expansion](design-decisions.md#source-40b141b58369810f8351fd6615b3a8a8)

**Status:** Current

**Area:** Domain model; Architecture; Repository mapping

**Previous version:** [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2)

**date:Last synced:start:** 2026-11-01

**date:Last synced:is_datetime:** 0

**Working copy:** https://app.notion.com/p/40b141b5836981568752c1b6a854ddf0

**Name:** Flowsheet Process Domain Model — v1.1

**Revision record** · 2026-11-01. The user authorized incorporation of the detailed improvements into the current Notion design and implementation plan. This record preserves the v1.1 adoption basis; the linked structured pages and catalog remain the live design authority.

#### Changes from v1

- Distinguish a material subject, selected characterization and model representation. Preserve exact chemistry guarantees and explicitly qualify empirical aggregate models.
- Make population distributions, polymer structure, weighting, internal coordinates and information loss explicit in material contracts.
- Separate flow, inventory, material lot, operating procedure, production campaign and computational Study.
- Specify use-specific substitutability and separate residual exposure, input/output evaluation and stateful external time advance.
- Carry scientific records, observations, cost/environmental boundaries and result coverage through existing owners.
- Make source editing, semantic identity, selected inspection scope and explicit recording precise for coding-agent delivery.

The catalog adds six semantic roles to the original 45: Material characterization; Population and structure representation; Material lot; Operating procedure; Production campaign; Engineering assessment. They describe contracts and views over authored identities, not a requirement for six new runtime classes or database tables.

#### Live design and delivery

[Flowsheet Process Domain Model](../process/index.md) owns the domain chapters and catalog.

[Implementation design and IDAES equivalence plan](../delivery/index.md) owns P0–P7 and the A–D first-delivery packets.

[Plan detailing and industry coverage assessment](plan-detailing-assessment.md) retains I01–I08, rationale, detailed contracts and the incorporation map.

The reusable overall framework remains v3. The industry horizon guides interface choices and finite later increments; candidate non-IDAES cases are not automatically added to the current qualification population.

#### Evidence boundary

The initial v1 review at repository revision 9266555 is preserved in [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2). This refinement uses the inspected [pse-arrow revision 19e7b49](https://github.com/paul-heyse/pse-arrow/tree/19e7b49a67df24c1bf73005a6186e145e930d53a) and the pinned IDAES/reference versions in the implementation plan.

This is a documentation and planning adoption. No repository implementation, build, numerical run or qualification campaign was performed by this update. Existing source-only, unattempted and historical evidence retain their meanings. This record is an index and revision rationale, not a frozen snapshot of every live page.


<a id="source-40b141b5836981ae9187c0bc93c10aeb"></a>

## Flowsheet implementation and IDAES equivalence plan — v1 — v1 (1 Nov 2026, adopted high-level plan)

Source: [Notion](https://app.notion.com/p/40b141b5836981ae9187c0bc93c10aeb).

**Summary:** Adopts the existing P0–P7 implementation program with I01–I08 refinements, A–D first-delivery packets, finite versioned IDAES qualification, and a separately scoped cross-industry expansion strategy. No implementation progress is claimed.

**Version:** v1 (1 Nov 2026, adopted high-level plan)

**Decisions:** [Cross-industry material models preserve characterization, representation and scientific evidence](design-decisions.md#source-40b141b58369810d9cdec738fceffbca); [Physical operating continuity and external execution capabilities remain explicit](design-decisions.md#source-40b141b58369814f9a9df88bd2f862d3); [Adopt bounded coding packets with finite IDAES qualification and separately scoped industry expansion](design-decisions.md#source-40b141b58369810f8351fd6615b3a8a8)

**Status:** Current

**Area:** Architecture; Repository mapping; Execution

**date:Last synced:start:** 2026-11-01

**date:Last synced:is_datetime:** 0

**Working copy:** https://app.notion.com/p/40b141b5836981b6a2b3f3e52d80ad13

**Name:** Flowsheet implementation and IDAES equivalence plan — v1

**Adopted high-level implementation plan** · 2026-11-01. The user authorized incorporating the assessment's improvements into the live Notion content. The plan fixes intended behavior, ownership, scientific assumptions and evidence obligations while delegating local implementation choices to the coding agent.

#### Plan owners

[Implementation design and IDAES equivalence plan](../delivery/index.md) is the live hub. Its six chapters own architecture, science, IDAES scope, qualification, sequencing and the source-inspected reference inventory.

[Coding-agent implementation brief](../delivery/coding-agent-brief.md) supplies the fixed/delegated boundary and precise source/API constraints. The roadmap incorporates its four packets: ordinary inputs, inspection/editing, complete native journeys and matched references.

#### Scope and implementation status

P0–P7 remains the delivery sequence. Full equivalence is a finite claim over selected released capabilities, configurations and versioned cases, with original physical checks and complete required evidence. Deliberate scientific improvements use their own comparison lane.

The broader process-industry horizon is adopted as design direction. Its representative cases expose semantic requirements; later non-IDAES implementation increments need explicit selection at the existing plan owner. They do not silently expand the IDAES denominator.

No code, build, simulation or qualification work is marked complete by this record. Existing repository change, conformance, native attempt and publication owners remain authoritative; ordinary implementation choices need no new approval cadence.

#### Evidence baseline

- pse-arrow: `19e7b49a67df24c1bf73005a6186e145e930d53a`.
- IDAES-PSE 2.13.0: `84532d7f99a9f976b05170948147afdd69cfa5f6`.
- IDAES/examples: `eb64e38ac429b7e3ae4f8db071f8d8cad2eef675`.
- IDAES compatibility: `8f1ae7a3ca358625128a78722bc2fdb75753a441`.

The plan links inspected source, exact qualifications and remaining compatibility uncertainty. The 55 documented notebook families are a research inventory; the earlier 12 workflows are architecture witnesses. Neither is a completed equivalence result.


<a id="source-40b141b583698133b0b5f31fc2d3a728"></a>

## Unified Simulation Domain Model — v3 — v3 (1 Nov 2026, process domain alignment)

Source: [Notion](https://app.notion.com/p/40b141b583698133b0b5f31fc2d3a728).

**Summary:** Adopts a flowsheet product domain over the general formulation framework; corrects task profiles, validation, fidelity, connections, bounds, identity and reconstruction while preserving established lifecycle decisions.

**Version:** v3 (1 Nov 2026, process domain alignment)

**Decisions:** [Bounds retain their provenance; the Query selects active enforcement](design-decisions.md#source-40a141b58369813eadace338d5d21224); [Versioned definitions are referenced by model instances](design-decisions.md#source-40a141b58369814084d2df8fc1e8654c); [Regime is analysis intent constrained by model capabilities](design-decisions.md#source-40a141b583698171ad14c4860b2f2d0e); [Flowsheet process modeling is the product domain; the overall model is its reusable framework](design-decisions.md#source-40b141b5836981a48565fc5bb95aa7f8); [Authored definitions remain the single executable model authority](design-decisions.md#source-40b141b5836981fda780eb758b00d078); [Units compose zero or more balance regions and submodels](design-decisions.md#source-40b141b5836981d28c9fe407531c5f38); [Material states, ports, streams and connections have distinct contracts](design-decisions.md#source-40b141b5836981a396e3c0bebf08f087); [Physical choices, equivalent formulations and numerical approximations remain distinct](design-decisions.md#source-40b141b58369811cba09ddd746667e97); [Answer kinds are extensible task profiles with composable requirements](design-decisions.md#source-40b141b5836981e2a8fcf88ce59fbe96); [Validation and scientific acceptance report qualified evidence](design-decisions.md#source-40b141b58369816cb748c04bfe69e5dc); [Stable semantic identity and qualified reconstruction preserve traceability](design-decisions.md#source-40b141b583698138a397edbd352c8966); [Process cases and operating sequences reuse the overall task lifecycle](design-decisions.md#source-40b141b583698197a991f73e23cf49e9); [Representative IDAES workflows establish domain and implementation coverage](design-decisions.md#source-40b141b5836981ed8963d64a2bc8062f)

**Status:** Current

**Area:** Domain model; Execution; Architecture

**Previous version:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b583698131bd16c51b551f1d9f)

**date:Last synced:start:** 2026-11-01

**date:Last synced:is_datetime:** 0

**Working copy:** https://app.notion.com/p/40a141b5836981c882fbf0449dfbcd7a

**Name:** Unified Simulation Domain Model — v3

> Revision record for the overall computational domain model. The structured hub and its databases are the live design authority; this page records the adopted changes and evidence. The v2 source snapshot remains preserved in the previous version.

#### Revision basis

Paul authorized application of the completed design review on 2026-11-01: update the current overall domain model and produce a dedicated Flowsheet Process Domain Model in Notion.

The overall model is a reusable formulation and execution framework. The process-facing domain owns flowsheet structure, material/state semantics, equipment, inventories, physical exchanges, operating cases and engineering requirements. Broad molecular, electronic-structure and general CFD examples remain extension illustrations rather than commitments for the process front end.

#### Adopted changes

1. Preserve acausal models, separate case data and analysis requests, immutable submissions, Study orchestration, physical typing, source provenance and engineering diagnostics.
2. Make the five answer kinds supported task profiles. Admit feasible questions, operating constraints and assessment-only observations according to declared formulation semantics and capabilities.
3. Separate physical-model choices, mathematical formulations and numerical approximations. Model substitution requires common exposed observables, conservation obligations and explicit applicability.
4. Replace universal pre-solve proof promises with staged structural, capability, numerical and scientific assessment. Unknown, unavailable and unsupported evidence remains explicit.
5. Give bounds and requirements their proper provenance: physical support, model validity, equipment ratings, case envelopes, decision bounds and numerical safeguards.
6. Treat regime as analysis intent checked against model capabilities, inventory definitions and available transformations.
7. Use semantic identities alongside readable paths. Preserve origin mappings and qualified reconstruction without promising every lowering has an exact inverse.
8. Distinguish stationary phase solutions, implemented stability checks and certified global equilibrium. Branch selection is semantic; starts alone are not branch contracts.
9. Strengthen connector contracts for transported material, explicit mixing/splitting, translations and optional reverse flow.
10. Keep authored definitions as the repository's model authority. Process views and packages reuse the existing identities and lowering path.

#### Preserved prior decisions

The Study remains above the atomic SimulationProblem. Infer remains first-class. Execution uses explicit kernels and plans. Numerical meshes remain execution choices for a selected physical model. Symbolic scientific knowledge remains preferred, with external evaluation through declared contracts. Structured choices are representable without implying every associated solver exists.

#### Evidence and implementation boundary

The review inspected [pse-arrow at 9266555](https://github.com/paul-heyse/pse-arrow/tree/926655517f6eac4ab52ae7a817a7b72168afdc25) and the existing Notion v2 design. The repository pins selected scientific comparisons to IDAES 2.13.0. Source-level evidence is separate from numerical qualification; no new parity campaign was executed for this design update.

Existing process-state specifications, material ports, contribution/inventory accounting, typed connections and lifecycle separation are retained. The domain target does not imply that broader multiphase, electrolyte, distributed dynamic or specialized equipment capabilities are already implemented.

#### Live design

[Unified Simulation Domain Model](../simulation/index.md)

#### Revision status

**Current** as of 2026-11-01. The 21 existing narrative pages and 143 existing catalog rows have been revised in place; three generic concepts were added, bringing the Concepts catalog to 100. The five Answer-kind page bodies also reflect the corrected profile contracts.

The companion [Flowsheet Process Domain Model](../process/index.md) is published with five chapters, 45 process concepts and 12 coverage cases. Its version record is [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).

The three previously open v2 decisions were resolved with qualifications, and ten adopted review decisions were recorded. Existing child pages and databases and the v2 historical source snapshot are preserved. This page is a revision record and index to the live design, not a frozen copy of every structured page or row.
