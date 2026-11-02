---
title: "Process concepts — Structure"
---

# Process concepts — Structure

<a id="source-40b141b5836981b49d9ff489fbf37e6b"></a>

## Index domain

Source: [Notion](https://app.notion.com/p/40b141b5836981b49d9ff489fbf37e6b).

**Cardinality:** Zero or more domains per model; one or more semantic axes per indexed occurrence.

**Invariants:** Physical stages, mesh nodes, time and scenarios are not interchangeable even when generic indexing is shared. Material internal coordinates and their numerical bins/moments are distinct from chemical species, physical spatial geometry and mesh resolution. Weighting, density versus amount and reduction closure belong to the selected representation.

**Area:** Structure

**Implementation:** Existing finite and continuous-domain machinery; target explicit process-facing distinction.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md

**Definition:** A declared membership or coordinate domain used to locate model quantities and relationships.

**Owner:** Owning definition or local submodel

**Relationships:** Includes component, phase, physical stage, spatial coordinate, time or scenario with distinct declared meaning.

**Concept:** Index domain

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).


<a id="source-40b141b5836981f2adf9eccf319ff9d0"></a>

## Model instance

Source: [Notion](https://app.notion.com/p/40b141b5836981f2adf9eccf319ff9d0).

**Cardinality:** One definition binding; zero or more child instances, quantities and ports.

**Invariants:** Instance identity is distinct from definition identity; reuse does not accidentally share numerical storage.

**Area:** Structure

**Implementation:** Existing checked specialization; target coherent process-facing ownership view.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md

**Definition:** One specialized occurrence of a reusable definition with explicit configuration and member identities.

**Owner:** Exactly one containing instance or selected root

**Relationships:** References a model definition; owns actual quantity, state and connection occurrences.

**Concept:** Model instance

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).


<a id="source-40b141b58369814faaeeeb8decb31779"></a>

## Model revision

Source: [Notion](https://app.notion.com/p/40b141b58369814faaeeeb8decb31779).

**Cardinality:** One revision contains one resolved dependency closure; many cases and runs may reference it.

**Invariants:** Execution cannot mutate the checked revision or substitute an unrecorded dependency. Text and declaration editing have an intentional authority policy. Declaration edits do not retain old document text as edited-IR authority. Presentation-label changes are distinct from contractual state-slot role or scientific-dependency edits.

**Area:** Structure

**Implementation:** Existing immutable model/package contract; retain as sole model-version authority.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/architecture-overview.md

**Definition:** Immutable admitted source package closure and the identities needed to reproduce its scientific meaning.

**Owner:** Model/package admission

**Relationships:** Binds definitions and declared data; referenced by operating cases, studies and results.

**Concept:** Model revision

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/architecture-overview.md).


<a id="source-40b141b5836981c18757d93e00569299"></a>

## Physical quantity

Source: [Notion](https://app.notion.com/p/40b141b5836981c18757d93e00569299).

**Cardinality:** One physical identity; zero or more case roles, aliases, observations and result values.

**Invariants:** Case role does not redefine physical meaning; display values and compiled coordinates remain projections.

**Area:** Structure

**Implementation:** Existing typed members and physical semantics; expose process roles without duplication.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md

**Definition:** An original scalar or indexed model member with physical kind, units, basis, datum and semantic subject.

**Owner:** Authored declaration and specialized occurrence

**Relationships:** Used by states, balances, closures, specifications, studies and evidence.

**Concept:** Physical quantity

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/physical-semantics.md).


<a id="source-40b141b583698159b69df7504292965d"></a>

## Process model definition

Source: [Notion](https://app.notion.com/p/40b141b583698159b69df7504292965d).

**Cardinality:** One definition; zero or more instances and child declarations.

**Invariants:** Executable meaning has one authored authority; a process catalog or UI is a view over the same identities.

**Area:** Structure

**Implementation:** Existing generic authored definitions; target process-facing contract and catalog view.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md

**Definition:** Reusable authored process composition with quantities, physical relationships, interfaces and permitted configuration.

**Owner:** Authored package definition

**Relationships:** Implemented by unit or subflowsheet definitions; referenced by model instances and revisions.

**Concept:** Process model definition

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).


<a id="source-40b141b5836981c8b922ec6f4a277c94"></a>

## Subflowsheet

Source: [Notion](https://app.notion.com/p/40b141b5836981c8b922ec6f4a277c94).

**Cardinality:** One containment owner; zero or more children and exposed boundary mappings.

**Invariants:** Containment is acyclic after finite specialization; hierarchy does not dictate a separate solve.

**Area:** Structure

**Implementation:** Existing wrappers and composition; target consistent exposed-interface semantics.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/campaign/models/recycle-flash.pse

**Definition:** A reusable composed process model whose selected child boundaries are exposed at a parent interface.

**Owner:** Containing model instance

**Relationships:** Special case of composition; shares quantity and port identities through explicit aliases/mappings.

**Concept:** Subflowsheet

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/campaign/models/recycle-flash.pse).


<a id="source-40b141b5836981e487f2cab14f54965f"></a>

## Unit model

Source: [Notion](https://app.notion.com/p/40b141b5836981e487f2cab14f54965f).

**Cardinality:** Zero or more ports, states, balance domains, closures and child models.

**Invariants:** No mandatory one-control-volume or one-inlet/one-outlet rule; unsupported configurations reject explicitly.

**Area:** Structure

**Implementation:** Existing authored unit library; target broadened contract proven by representative workflows.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse

**Definition:** A process component exposing an engineering interface and implementing physical behavior through composition.

**Owner:** Authored model definition and its instances

**Relationships:** May contain control volumes, custom balances, interfaces, property states or subordinate units.

**Concept:** Unit model

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/process/models/heat-exchanger.pse).
