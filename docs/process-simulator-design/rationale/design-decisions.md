---
title: "Design Decisions"
---

# Design Decisions

<a id="source-40b141b58369810f8351fd6615b3a8a8"></a>

## Adopt bounded coding packets with finite IDAES qualification and separately scoped industry expansion

Source: [Notion](https://app.notion.com/p/40b141b58369810f8351fd6615b3a8a8).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f); [Flowsheet implementation and IDAES equivalence plan — v1](version-records.md#source-40b141b5836981ae9187c0bc93c10aeb)

**Question:** How should the assessment become actionable for the coding agent while preserving engineering autonomy, exact evidence and the larger industry ambition?

**Status:** Decided

**Decision:** Adopt bounded coding packets with finite IDAES qualification and separately scoped industry expansion

Adopted through the user's instruction to incorporate the detailed improvements on 2026-11-01.

The existing implementation plan adopts I01–I08 and retains P0–P7. P1 is detailed into four packets: ordinary inputs, inspection/editing, native journeys and matched independent references. Packets fix engineering behavior, scientific assumptions, ownership and discriminating evidence; internal organization and routine implementation decisions remain delegated.

Source/declaration editing, contractual identity, selected inspection scope, numeric versus structural/scientific edits and explicit recording follow the existing repository owners. Tests challenge the actual boundary and failure reason. A missing prerequisite for a selected required reference invocation is a failure; genuinely unrun scope stays unattempted.

The cross-industry semantic direction is adopted now. Representative non-IDAES cases remain candidate finite increments to select at the plan owner; this decision does not add all industries to P1 or to the IDAES qualification denominator. Accounting boundaries and observation coverage remain explicit.

See [Implementation design and IDAES equivalence plan](../delivery/index.md), [Coding-agent implementation brief](../delivery/coding-agent-brief.md) and [Industry coverage and expansion strategy](../delivery/industry-coverage-and-expansion.md). Documentation adoption is complete only when the linked owners are reconciled; it does not mark implementation or qualification complete.


<a id="source-40b141b5836981e2a8fcf88ce59fbe96"></a>

## Answer kinds are extensible task profiles with composable requirements

Source: [Notion](https://app.notion.com/p/40b141b5836981e2a8fcf88ce59fbe96).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** Are Solve, Extremize, Expect, Infer and Analyze a closed exhaustive taxonomy?

**Status:** Decided

**Decision:** Answer kinds are extensible task profiles with composable requirements

#### Adopted resolution

Keep the five supported profiles, with Infer first class, while defining eligibility through explicit intent, roles, data and capabilities. Solve may include operating constraints and assessment-only observations. Structural analysis need not follow a successful solve. New or composed profiles require explicit semantics and supported products; a label alone grants no execution capability.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40b141b5836981fda780eb758b00d078"></a>

## Authored definitions remain the single executable model authority

Source: [Notion](https://app.notion.com/p/40b141b5836981fda780eb758b00d078).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** Should a process-facing model introduce a second object graph or equation-building authority?

**Status:** Decided

**Decision:** Authored definitions remain the single executable model authority

#### Adopted resolution

Use process contracts, authored packages, roles and views over existing definition, instance, quantity, interface and connection identities. The process design guides the front end and scientific model libraries; it does not create an independent equipment database that owns duplicate equations or state. Change the generic kernel only for a demonstrated semantic gap.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40a141b58369813eadace338d5d21224"></a>

## Bounds retain their provenance; the Query selects active enforcement

Source: [Notion](https://app.notion.com/p/40a141b58369813eadace338d5d21224).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b583698131bd16c51b551f1d9f); [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** Confirm: should decision bounds and operating limits live in the Query (admissible sets, query constraints) rather than the Specification?

**Status:** Decided

**Decision:** Bounds retain their provenance; the Query selects active enforcement

Introduced in v2. Physical bounds stay on Model quantities; decision bounds become admissible sets; safety and operating limits become query constraints. Flagged by the revision as a change worth checking.

#### Resolution — overall model v3

**Decided** on 2026-11-01 as part of the authorized domain-model revision.

The blanket proposal that all bounds move to the Query is refined. Bounds and requirements retain their scientific or engineering owner; the active Query selects how task-relevant conditions are enforced or assessed.

| Meaning | Owner |
| --- | --- |
| Physical support and defining admissibility | Model / typed quantity contract |
| Correlation and property validity | Selected property or constitutive model |
| Equipment ratings and configuration limits | Equipment definition/configuration |
| Operating envelopes and their source | Operating case / Specification |
| Active operating and design requirements | Query, with enforce / assess / advisory semantics |
| Decision domain | Query |
| Numerical safeguarding and step limits | Execution / MethodRequest, separately identified |

A numerical guard must not silently redefine a physical feasible region. A measured value or uncertainty distribution is knowledge, not automatically an enforced bound.

The original v2 question and introduction above are retained as history. Applied in [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40b141b58369810d9cdec738fceffbca"></a>

## Cross-industry material models preserve characterization, representation and scientific evidence

Source: [Notion](https://app.notion.com/p/40b141b58369810d9cdec738fceffbca).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f); [Flowsheet implementation and IDAES equivalence plan — v1](version-records.md#source-40b141b5836981ae9187c0bc93c10aeb)

**Question:** How should broader industrial materials and model substitution fit the flowsheet domain without weakening exact chemistry or making seed-state conventions universal?

**Status:** Decided

**Decision:** Cross-industry material models preserve characterization, representation and scientific evidence

Adopted through the user's instruction to incorporate the detailed improvements on 2026-11-01.

Material subject identity, selected characterization, chemical/model components, material representation, population structure and measured quality retain distinct meanings. Exact chemical mappings and reactions keep their composition/charge requirements. Empirical aggregates declare their justified balances, approximation, information loss and applicability.

Model substitution is assessed for the requested observables, material/state meaning, validity, derivatives, initialization and effects. Compatible ports establish composability, not scientific equivalence.

This incorporates I03, I04 and the scientific-data portions of I07. See [Materials, states, and physical models](../process/materials-states-and-physical-models.md) and [Cross-industry domain contracts](../process/cross-industry-contracts.md). It extends the earlier material/port decision without replacing the authored-model authority or asserting new implemented science.


<a id="source-40a141b5836981ad9bf4dbf3e8bbbd78"></a>

## Execution is a set of translation kernels, not a single router

Source: [Notion](https://app.notion.com/p/40a141b5836981ad9bf4dbf3e8bbbd78).

**date:Decided on:start:** 2026-10-31

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b58369812b9f43eb8609631238)

**Question:** How should execution translate a problem into solver form?

**Status:** Decided

**Decision:** Execution is a set of translation kernels, not a single router

Each kernel lowers one canonical form to another. The API section is being revised to match in v2.


<a id="source-40b141b5836981a48565fc5bb95aa7f8"></a>

## Flowsheet process modeling is the product domain; the overall model is its reusable framework

Source: [Notion](https://app.notion.com/p/40b141b5836981a48565fc5bb95aa7f8).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** Should the front end target a universal simulation ontology or a coherent flowsheet process domain?

**Status:** Decided

**Decision:** Flowsheet process modeling is the product domain; the overall model is its reusable framework

#### Adopted resolution

Focus implementation on flowsheet process modeling, including detailed local equipment physics where needed. Retain the general formulation/execution framework as reusable infrastructure. General CFD, molecular simulation and electronic structure remain explicitly scoped integrations. Catalog breadth does not constitute a product or backend support promise.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40a141b5836981a98488ca181dbcd28e"></a>

## Infer is a first-class answer kind

Source: [Notion](https://app.notion.com/p/40a141b5836981a98488ca181dbcd28e).

**date:Decided on:start:** 2026-10-31

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b58369812b9f43eb8609631238)

**Question:** Is Infer a first-class answer kind?

**Status:** Decided

**Decision:** Infer is a first-class answer kind

It lowers to Extremize or Expect, but its observation model, noise model, identifiability check and covariance result get their own contract.


<a id="source-40b141b5836981a396e3c0bebf08f087"></a>

## Material states, ports, streams and connections have distinct contracts

Source: [Notion](https://app.notion.com/p/40b141b5836981a396e3c0bebf08f087).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** What physical meaning belongs to material state, boundary exposure and connectivity?

**Status:** Decided

**Decision:** Material states, ports, streams and connections have distinct contracts

#### Adopted resolution

Material state, transported observations and stored inventory are distinct. A port exposes existing quantities; a connection couples admitted endpoints; a stream is a process-facing view of the identified connection or boundary. Mixing, splitting, translation, storage and physical line behavior have explicit equations. Reverse flow requires declared transport semantics; allowing negative flow is insufficient.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40a141b583698193907ed1fa9590cc03"></a>

## Next pass makes every conceptual component and its hierarchy explicit

Source: [Notion](https://app.notion.com/p/40a141b583698193907ed1fa9590cc03).

**date:Decided on:start:** 2026-10-31

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b58369812b9f43eb8609631238)

**Question:** What should the deeper revision focus on?

**Status:** Decided

**Decision:** Next pass makes every conceptual component and its hierarchy explicit

Conceptual components and their place in the hierarchy, not physical data shapes.


<a id="source-40a141b58369817a844cdefc929ea71a"></a>

## One Expect covers input uncertainty and intrinsic stochasticity

Source: [Notion](https://app.notion.com/p/40a141b58369817a844cdefc929ea71a).

**date:Decided on:start:** 2026-10-31

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b58369812b9f43eb8609631238)

**Question:** Is there one Expect for input uncertainty and intrinsic stochasticity?

**Status:** Decided

**Decision:** One Expect covers input uncertainty and intrinsic stochasticity

Both integrate over a measure; they differ only in where the measure comes from.


<a id="source-40b141b58369811cba09ddd746667e97"></a>

## Physical choices, equivalent formulations and numerical approximations remain distinct

Source: [Notion](https://app.notion.com/p/40b141b58369811cba09ddd746667e97).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** May model fidelity, mathematical form and discretization share one undifferentiated alternatives mechanism?

**Status:** Decided

**Decision:** Physical choices, equivalent formulations and numerical approximations remain distinct

#### Adopted resolution

Record different physical assumptions separately from mathematically equivalent representations and numerical approximation. Alternatives advertise only the observables and capabilities they actually share, together with validity and conservation obligations. A lumped model cannot promise a distributed field without an explicit justified mapping. Every substitution and lowering retains its assumptions and source identity.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40b141b58369814f9a9df88bd2f862d3"></a>

## Physical operating continuity and external execution capabilities remain explicit

Source: [Notion](https://app.notion.com/p/40b141b58369814f9a9df88bd2f862d3).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f); [Flowsheet implementation and IDAES equivalence plan — v1](version-records.md#source-40b141b5836981ae9187c0bc93c10aeb)

**Question:** How should lots, procedures, campaigns and external dynamic models compose without confusing physical continuity with computational orchestration?

**Status:** Decided

**Decision:** Physical operating continuity and external execution capabilities remain explicit

Adopted through the user's instruction to incorporate the detailed improvements on 2026-11-01.

Equipment, stored inventories, material lots, physical procedures/campaigns and computational Studies retain their separate owners. Transitions declare retained states, additions/removals, resets and resources. Independent Study scenarios do not imply physical carryover. Prescribed schedule execution, conflict checking and schedule optimization are separate capabilities.

External residual exposure, input/output evaluation and stateful time advance are distinct operations. Time advance requires explicit state custody, interval/event behavior and actual restoration/derivative capabilities; repeated residual evaluations must not silently advance an external model.

This incorporates I05 and I06. See [Cases, specifications, and studies](../process/cases-specifications-and-studies.md) and [Cross-industry domain contracts](../process/cross-industry-contracts.md). These adopted contracts do not claim arbitrary hybrid dynamics, a production scheduler or co-simulation is currently implemented.


<a id="source-40b141b583698197a991f73e23cf49e9"></a>

## Process cases and operating sequences reuse the overall task lifecycle

Source: [Notion](https://app.notion.com/p/40b141b583698197a991f73e23cf49e9).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** How do process cases, atomic analyses, physical steps and Study orchestration relate?

**Status:** Decided

**Decision:** Process cases and operating sequences reuse the overall task lifecycle

#### Adopted resolution

Map an operating case to Specification and an atomic engineering question to Query within SimulationProblem. Study coordinates explicit atomic tasks; jointly fitted shared parameters can couple several experiments inside one Infer problem. Physical operating steps/events are model and case semantics. Numerical initialization, plans, retries and actual Runs retain separate identities and lifecycles.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40a141b583698171ad14c4860b2f2d0e"></a>

## Regime is analysis intent constrained by model capabilities

Source: [Notion](https://app.notion.com/p/40a141b583698171ad14c4860b2f2d0e).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b583698131bd16c51b551f1d9f); [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** Confirm: is steady versus transient versus periodic a choice of question rather than of model?

**Status:** Decided

**Decision:** Regime is analysis intent constrained by model capabilities

Introduced in v2 as an optional Query entry per evolution coordinate, orthogonal to every answer kind. Flagged by the revision as a change worth checking.

#### Resolution — overall model v3

**Decided** on 2026-11-01 as part of the authorized domain-model revision.

Steady, transient and periodic requests are Query intent. The selected Model must provide the physical capabilities and admitted transformations required to realize that intent.

Transient storage needs explicit inventories and accumulation laws; a Query cannot create missing storage physics. Memoryless units may remain algebraic inside a dynamic flowsheet. A steady reduction of a dynamic model and a periodic boundary condition each retain their declared assumptions and consistency requirements. Holdup can matter in steady operation and is not synonymous with a transient regime.

Physical operating sequences and events remain process semantics. Numerical time stepping and initialization remain execution choices.

The original v2 question and introduction above are retained as history. Applied in [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40b141b5836981ed8963d64a2bc8062f"></a>

## Representative IDAES workflows establish domain and implementation coverage

Source: [Notion](https://app.notion.com/p/40b141b5836981ed8963d64a2bc8062f).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** What evidence should determine whether the process domain is ready for broad IDAES replication?

**Status:** Decided

**Decision:** Representative IDAES workflows establish domain and implementation coverage

#### Adopted resolution

Use concrete acceptance cases spanning recycle, exchanger role changes, translation, reactions, dynamics/control, distributed and multi-stream equipment, electrolytes, solids, cyclic operation and the SOC integration boundary. Pin reference versions and align scientific assumptions before numerical comparison. Record source evidence, design intent, executed checks and parity separately; a conceptual catalog entry is not implementation evidence.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40a141b58369815faef9dc7af6fa206d"></a>

## SimulationProblem stays atomic; the Study is a layer above

Source: [Notion](https://app.notion.com/p/40a141b58369815faef9dc7af6fa206d).

**date:Decided on:start:** 2026-10-31

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b58369812b9f43eb8609631238)

**Question:** Is the Study inside the single API object?

**Status:** Decided

**Decision:** SimulationProblem stays atomic; the Study is a layer above

The Study is a thin layer above SimulationProblem, so execution has one unit of work.


<a id="source-40b141b583698138a397edbd352c8966"></a>

## Stable semantic identity and qualified reconstruction preserve traceability

Source: [Notion](https://app.notion.com/p/40b141b583698138a397edbd352c8966).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** Are readable paths sufficient identity, and must every lowering have a total inverse?

**Status:** Decided

**Decision:** Stable semantic identity and qualified reconstruction preserve traceability

#### Adopted resolution

Use stable typed semantic identifiers alongside readable paths, revisions and explicit origin maps. Preserve identity across supported renames and instance composition. Reconstruction may be partial, approximate or unavailable after elimination, aggregation, discretization or relaxation. Values, duals, derivatives and error evidence are mapped back only with their justified meaning.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40a141b5836981c9a389f2cab9cd4224"></a>

## Structured decision spaces are in scope as a value type; solvers later

Source: [Notion](https://app.notion.com/p/40a141b5836981c9a389f2cab9cd4224).

**date:Decided on:start:** 2026-10-31

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b58369812b9f43eb8609631238)

**Question:** Are structured decision spaces in scope?

**Status:** Decided

**Decision:** Structured decision spaces are in scope as a value type; solvers later

Yes, as a Decision value type (for example a molecular graph). Solvers for them are a later extension.


<a id="source-40a141b5836981b3a336c583de758998"></a>

## Symbolic relations by default; opaque only where knowledge exists only as code

Source: [Notion](https://app.notion.com/p/40a141b5836981b3a336c583de758998).

**date:Decided on:start:** 2026-10-31

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b58369812b9f43eb8609631238)

**Question:** How strongly are symbolic relations preferred?

**Status:** Decided

**Decision:** Symbolic relations by default; opaque only where knowledge exists only as code

Symbolic wherever the physics is known; opaque only for quantum codes, legacy property packages and learned surrogates.


<a id="source-40a141b58369814288c3e94bdb793ff8"></a>

## The mesh is execution, not part of the problem

Source: [Notion](https://app.notion.com/p/40a141b58369814288c3e94bdb793ff8).

**date:Decided on:start:** 2026-10-31

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b58369812b9f43eb8609631238)

**Question:** Is a CFD mesh part of the problem?

**Status:** Decided

**Decision:** The mesh is execution, not part of the problem

The mesh is a method choice recorded in provenance; the accuracy demand carries the user's intent. A MethodRequest entry marked "required" covers cases where a specific mesh must be used.


<a id="source-40b141b5836981d28c9fe407531c5f38"></a>

## Units compose zero or more balance regions and submodels

Source: [Notion](https://app.notion.com/p/40b141b5836981d28c9fe407531c5f38).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** Must every unit inherit exactly one standard control volume?

**Status:** Decided

**Decision:** Units compose zero or more balance regions and submodels

#### Adopted resolution

No universal control-volume ancestry or fixed inlet/outlet count is imposed. A unit may contain zero, one or many balance regions, state submodels, constitutive closures and children. Translators, multi-stream contactors, exchangers and detailed electrochemical equipment motivate this composition. Physical stages and geometry are retained as model structure; mesh points remain numerical realization.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40b141b58369816cb748c04bfe69e5dc"></a>

## Validation and scientific acceptance report qualified evidence

Source: [Notion](https://app.notion.com/p/40b141b58369816cb748c04bfe69e5dc).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** Does structural admission prove arbitrary well-posedness or scientific correctness?

**Status:** Decided

**Decision:** Validation and scientific acceptance report qualified evidence

#### Adopted resolution

Separate definite semantic/type violations, structural diagnostics under assumptions, local numerical evidence and actual certificates. Zero degree of freedom is not a proof of existence or uniqueness. Solver convergence is not physical acceptance. Gibbs stationarity is not automatically stable global equilibrium. Required unsupported claims are reported; optional unavailable products are withheld with reasons.

#### Basis and consequences

Adopted on 2026-11-01 through Paul's instruction to apply the completed domain-model review and publish both models in Notion. This records the reviewed design direction; implementation and scientific qualification remain separate work.

The detailed definitions, qualifications and primary-source evidence are maintained in [Unified Simulation Domain Model](../simulation/index.md) and [Flowsheet Process Domain Model](../process/index.md).

Version records: [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).


<a id="source-40a141b58369814084d2df8fc1e8654c"></a>

## Versioned definitions are referenced by model instances

Source: [Notion](https://app.notion.com/p/40a141b58369814084d2df8fc1e8654c).

**date:Decided on:start:** 2026-11-01

**date:Decided on:is_datetime:** 0

**Document:** [A Unified Domain Model for Chemical Simulation](version-records.md#source-40a141b583698131bd16c51b551f1d9f); [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728); [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2); [Flowsheet Process Domain Model — v1.1](version-records.md#source-40b141b5836981958b9ad5a14519520f)

**Question:** Confirm: should every reusable definition (species, port types, constitutive models, component templates) live in the Vocabulary?

**Status:** Decided

**Decision:** Versioned definitions are referenced by model instances

Introduced in v2. The Model becomes a tree of component instances built from Vocabulary templates. Flagged by the revision as a change worth checking.

#### Resolution — overall model v3

**Decided** on 2026-11-01 as part of the authorized domain-model revision.

Retain the distinction between reusable definitions and instantiated occurrences. Vocabulary provides versioned definitions and references to libraries/packages; the Model contains the selected instantiated structure and its physical relationships.

This does not require a second repository holding every definition, copying all definitions into a submission, or a separate process equipment object graph. Authored package definitions remain the executable authority. A problem pins its resolved definition closure and selected realizations. Shared definitions are references; concrete instance quantities have one containment owner unless explicit sharing or aliasing semantics apply.

The original v2 question and introduction above are retained as history. Applied in [Unified Simulation Domain Model — v3](version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](version-records.md#source-40b141b583698185aee7fac52f7213f2).
