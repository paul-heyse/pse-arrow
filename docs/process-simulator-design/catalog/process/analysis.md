---
title: "Process concepts — Analysis"
---

# Process concepts — Analysis

<a id="source-40b141b583698189b7edc786b4da6976"></a>

## Engineering assessment

Source: [Notion](https://app.notion.com/p/40b141b583698189b7edc786b4da6976).

**Cardinality:** Zero or more assessments may reuse one qualified physical result; each defines its required observables, scope and time basis.

**Invariants:** Physical inventories remain distinct from prices, emissions factors, utilization, geography/period, allocation rules and functional units. Imported/exported utilities and internal transfers are counted once at the declared boundary. Missing horizon coverage cannot be silently annualized; arithmetic agreement is not applicability qualification.

**Area:** Analysis

**Implementation:** Adopted v1.1 design contract. Extend authored costing/accounting and current observation/result selection; no second analysis engine or assessment database. Existing exchanger costing remains a scoped arithmetic witness.

**Source URL:** https://app.notion.com/p/40b141b5836981bca17dde73c867160c

**Definition:** A case/study-level accounting interpretation of selected physical results, such as equipment costing, operating economics or environmental impact, with explicit boundaries and assumptions.

**Owner:** Authored assessment definitions and the existing Specification/Query/Study/result owners

**Relationships:** Study; Operating case; Production campaign; Property parameterization; Result and observation; Observation mapping; Physical quantity.

**Concept:** Engineering assessment

Adopted design contract in [Flowsheet Process Domain Model](../../process/index.md). This is a semantic role over existing authored identities; it does not prescribe a one-to-one runtime class or persistence record. The structured properties contain the definition and invariants.

Owning narrative: [Cross-industry domain contracts](../../process/cross-industry-contracts.md).

Implementation and scientific qualification remain separate from documentation adoption.


<a id="source-40b141b58369819ebfcbfa92f2929abd"></a>

## Numerical plan

Source: [Notion](https://app.notion.com/p/40b141b58369819ebfcbfa92f2929abd).

**Cardinality:** One effective selection per run or explicit workflow step; zero or more derived products.

**Invariants:** Numerical selection cannot weaken original physical obligations; tears and warm starts are not plant structure.

**Area:** Analysis

**Implementation:** Existing compiler/runtime owners; preserve separation from authored physical meaning.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-runtime/src/workflow/modeling/cases.rs

**Definition:** An admitted selection of preparation, discretization, initialization, scaling and solver behavior.

**Owner:** Analysis/runtime preparation

**Relationships:** Binds an atomic problem and its model/case/Query, with parent Study context where applicable; selects numerical routes according to actual capabilities.

**Concept:** Numerical plan

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-runtime/src/workflow/modeling/cases.rs).


<a id="source-40b141b58369817f9621c700b88c7802"></a>

## Observation mapping

Source: [Notion](https://app.notion.com/p/40b141b58369817f9621c700b88c7802).

**Cardinality:** One or more observations per selected dataset; each has an explicit modeled target or declared aggregation.

**Invariants:** A measurement is not automatically a hard specification; missing/uncertain data is not invented. A collection-interval average is not automatically a point state. Acquisition time maps explicitly to process time; phase/component/population basis and sampling aggregation are retained. Sensor/lot tags reference selected occurrences; conceptual designs need no fictitious installed asset.

**Area:** Analysis

**Implementation:** Existing fitting/data mechanisms; target complete process semantics for measurements.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/how_to_guides/workflow/data_rec_parmest.html

**Definition:** A mapping from measured evidence to an original physical quantity, location, time and error interpretation.

**Owner:** Experiment, dataset or study

**Relationships:** References units, basis, datum, sampling/averaging and uncertainty or weighting. Uses Material characterization, Material lot and Population and structure representation where relevant; separates fitted-data use from independent prediction evidence.

**Concept:** Observation mapping

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/how_to_guides/workflow/data_rec_parmest.html).


<a id="source-40b141b5836981b285b3f011e30a447e"></a>

## Study

Source: [Notion](https://app.notion.com/p/40b141b5836981b285b3f011e30a447e).

**Cardinality:** One or more atomic problem nodes and their case/scenario selections; zero or more data edges, generators, objectives and shared-quantity couplings as appropriate.

**Invariants:** Labels are profiles, not mutually exclusive universal categories; concrete capability requirements govern admission. Computational dependencies do not establish physical transfers or carryover between successive batches. Prescribed schedule simulation, resource-conflict checking and schedule optimization require different capabilities.

**Area:** Analysis

**Implementation:** Existing analyses and fit workflows; target consistent process-facing composition.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/workflows-and-results.md

**Definition:** The process-facing specialization of the overall Study: orchestration of one or more atomic problem/Query requests over selected operating cases.

**Owner:** Study authoring/runtime workflow

**Relationships:** Includes simulation, rating, sizing, optimization, sensitivity, uncertainty and fitting profiles. A joint multi-experiment fit can be one Infer problem with several scenarios; separate nodes require explicit coupling. May analyze an Operating procedure or Production campaign and select Engineering assessment assumptions without owning their physical equations.

**Concept:** Study

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/workflows-and-results.md).


<a id="source-40b141b5836981a1928ec1da9ed36e66"></a>

## Study scenario or experiment

Source: [Notion](https://app.notion.com/p/40b141b5836981a1928ec1da9ed36e66).

**Cardinality:** One case/model binding per scenario; zero or more observations and shared parameter links.

**Invariants:** Scenario identity is not physical time; shared and local parameters are explicitly distinguished.

**Area:** Analysis

**Implementation:** Existing steady/transient/mixed fitting basis; target clear domain ownership.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/how_to_guides/workflow/data_rec_parmest.html

**Definition:** One case selection within a study with explicit local conditions and shared-quantity coupling.

**Owner:** Study

**Relationships:** Participates in parameter fitting, sweeps, optimization or uncertainty ensembles.

**Concept:** Study scenario or experiment

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Cases, specifications, and studies](../../process/cases-specifications-and-studies.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/how_to_guides/workflow/data_rec_parmest.html).
