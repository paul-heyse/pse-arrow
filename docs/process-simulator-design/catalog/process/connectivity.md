---
title: "Process concepts — Connectivity"
---

# Process concepts — Connectivity

<a id="source-40b141b5836981719761e6296d3d52aa"></a>

## Connection occurrence

Source: [Notion](https://app.notion.com/p/40b141b5836981719761e6296d3d52aa).

**Cardinality:** Two endpoints per direct connection; many distinct parallel occurrences are permitted.

**Invariants:** Incidence and physical contracts are checked; directed flow is not mathematical evaluation order.

**Area:** Connectivity

**Implementation:** Existing process connections and flow projection; reversible semantics remain a separate capability.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md

**Definition:** One explicitly authored coupling between active compatible port occurrences.

**Owner:** Containing process composition

**Relationships:** Creates declared coordinate relationships, retains transport checks, and contributes explicit topology.

**Concept:** Connection occurrence

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).


<a id="source-40b141b583698169be35fee58d438822"></a>

## Exposed boundary mapping

Source: [Notion](https://app.notion.com/p/40b141b583698169be35fee58d438822).

**Cardinality:** Zero or more mappings; each parent boundary resolves to explicit child meaning.

**Invariants:** Cannot erase incidence limits, duplicate reconstruction or double-count transfer.

**Area:** Connectivity

**Implementation:** Existing composition/wrappers; target verified behavior under nesting and aliasing.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/campaign/models/recycle-flash.pse

**Definition:** An alias or declared transformation exposing child interfaces at a parent model boundary.

**Owner:** Subflowsheet or composite unit

**Relationships:** Connects hierarchy to ports while retaining original quantities and occurrence provenance.

**Concept:** Exposed boundary mapping

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/packages/reference/campaign/models/recycle-flash.pse).


<a id="source-40b141b5836981199468c8a16069131e"></a>

## Material or state translator

Source: [Notion](https://app.notion.com/p/40b141b5836981199468c8a16069131e).

**Cardinality:** One or more input/output state boundaries; zero or more mapping and conservation relationships.

**Invariants:** Mapping completeness, physical conservation and any information loss are explicit; no name-based conversion. Exact chemical mappings retain their complete composition/charge requirements. Empirical mappings state available conservation evidence and uncertainty; mass agreement does not imply element or charge closure. Target/source conservation projections must agree for each claimed quantity over the mapping's declared domain.

**Area:** Connectivity

**Implementation:** Existing translator contract; concrete package combinations need authored implementations and checks.

**Source URL:** https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/translator.html

**Definition:** An ordinary authored model that maps incompatible material/state conventions with explicit relationships.

**Owner:** Process composition or unit definition

**Relationships:** Connects material systems, bases, reference datums or state representations. Uses Material characterization, Composition representation and, where relevant, Population and structure representation.

**Concept:** Material or state translator

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/translator.html).


<a id="source-40b141b5836981dba943df528c7e01d3"></a>

## Material port

Source: [Notion](https://app.notion.com/p/40b141b5836981dba943df528c7e01d3).

**Cardinality:** Exactly one state specification per occurrence; connection count governed by authored incidence.

**Invariants:** Semantic coverage and physical compatibility determine admission, not package identity or display order. Composition/transport coordinates, reference conventions and reversal behavior are explicit. Interface compatibility is a composition condition, not a claim of equivalent internal physics, derivatives or dynamic memory.

**Area:** Connectivity

**Implementation:** Existing MaterialPort specialization and connection admission.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs

**Definition:** A material boundary aliasing one admitted state specification and its transported meaning.

**Owner:** Unit or subflowsheet boundary

**Relationships:** References material state coordinates and transport observations; participates in directed connection.

**Concept:** Material port

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs).


<a id="source-40b141b5836981f98642e4c90b9298b5"></a>

## Process stream

Source: [Notion](https://app.notion.com/p/40b141b5836981f98642e4c90b9298b5).

**Cardinality:** One identified connection or boundary per view; zero or more derived state and transport observations.

**Invariants:** Preserve original semantic identity, basis, species membership and energy reference. A display stream adds no hidden storage, pressure loss, mixing or heat transfer. Required physical behavior belongs to an explicit link model or unit. Stream flow is distinct from Inventory, Material lot and a collected product interval. Flow reversal requires explicit upstream composition/enthalpy and zero-flow semantics; a sign change alone is insufficient.

**Area:** Connectivity

**Implementation:** Proposed process-facing view over current ports, states and connection identities. Any persistent presentation record must reference those owners and must not become a second executable state authority.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md

**Definition:** A process-facing view of material passing through an identified connection or process boundary, with state and transport observations resolved from the admitted endpoint contract. It is not a second independently specified material state.

**Owner:** Flowsheet presentation/reporting view referring to the original model connection or boundary.

**Relationships:** Connection occurrence; Material port; Material state; Operating case; Result and observation. May expose population/structure observations supported by the admitted boundary; integrated amount or quality uses an explicit Observation mapping.

**Concept:** Process stream

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).


<a id="source-40b141b583698113ae37cde9fe15b427"></a>

## Typed port

Source: [Notion](https://app.notion.com/p/40b141b583698113ae37cde9fe15b427).

**Cardinality:** Zero or more per instance; aliases zero or more existing members under a declared interface.

**Invariants:** Port aliasing creates no duplicate numerical storage or automatic conservation claim.

**Area:** Connectivity

**Implementation:** Existing scalar/material port machinery; package vocabulary supplies physical roles.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md

**Definition:** A boundary view over existing typed quantities with explicit ownership, role and incidence policy.

**Owner:** Exactly one model boundary occurrence

**Relationships:** Used by direct connections, exposed-boundary mappings and process views.

**Concept:** Typed port

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Process structure and connectivity](../../process/structure-and-connectivity.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/models-and-composition.md).
