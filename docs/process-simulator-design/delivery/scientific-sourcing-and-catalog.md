---
title: "Scientific sourcing and catalog architecture — review and proposed detail"
notion_source: "https://app.notion.com/p/40b141b5836981d0866aff89e54192fe"
notion_last_edited: "2026-11-02T05:11:04.020Z"
---

# Scientific sourcing and catalog architecture — review and proposed detail

Source: [Notion](https://app.notion.com/p/40b141b5836981d0866aff89e54192fe). Imported 2026-11-02.

**Status: Native Rust/SurrealDB direction confirmed; implementation detail proposed** · 2026-11-01.

This revision incorporates the user's clarification: targeted scientific sourcing belongs within pse-arrow, using its Rust codebase and existing SurrealDB instance. The earlier proposed thermo-knowledge dependency is superseded. Unit-model structure was intentionally reserved for this domain-design exercise. Detailed composition and delivery choices below remain proposed implementation guidance.

Companion to [Implementation design and IDAES equivalence plan](index.md) and [Scientific packages and equipment delivery](scientific-packages-and-equipment.md). This chapter addresses systematic scientific sourcing and a queryable catalog. It records a proposed implementation direction; it does not change adopted version records or declare acquisition, native implementation or qualification complete.

## 1. Assessment and recommendation

The existing plan is strong about scientific meaning, provenance, physical admission and qualification. It is less explicit about how a coding agent discovers source material, extracts it reproducibly, chooses among incompatible formulations and datasets, and publishes a discoverable scientific inventory. The existing Process concepts catalog describes the domain vocabulary; its rows are not an inventory of actual equations, coefficient sets, unit configurations or experimental datasets.

Make each reusable scientific definition, dataset, parameterization, unit definition and supported configuration identifiable, versioned and queryable, with typed relationships to its sources, dependencies, implementations and evidence. Reuse the scientific owners already present. Treat discovery as a view over those authorities, with declared source coverage.

**pse-arrow owns the complete targeted scientific sourcing, admission, storage and discovery path.** Reuse its existing Rust workflows, authored scientific types, immutable source revisions, schema generation, canonical SurrealDB and research/result owners. External libraries and publications supply selected scientific content. There is no required thermo-knowledge service, PostgreSQL store, accepted-knowledge revision or cross-repository handoff.

The scope is the smallest complete scientific dependency set needed by the committed process cases, expanding as those cases demand. The breadth of a reference library does not determine the import population. Unit models are defined through the process domain's reusable physical structures, constitutive closures and engineering templates.

This is relevant to thermodynamics, transport, reaction kinetics and equilibrium, transfer correlations, hydraulics, solids and population models, empirical material characterizations, equipment closures, and scoped engineering assessments. Those families can enter progressively through real process cases; no universal physics framework is required.

## 2. Verified foundations and current limits

| Existing owner | What the reviewed source establishes | Consequence for this proposal |
| --- | --- | --- |
| Authored scientific declarations | Scientific equations, quantities, records, property/reaction contracts and unit definitions; source text and generated declaration values share admission | Imports can construct typed native declarations/data directly |
| Native source editing and admission | SourceDelta and SourceDocumentDelta carry additions, replacements, removals and data changes against an exact parent | Reuse ordinary contextual checking and source publication |
| Canonical source storage | Immutable source objects, bounded staging, closed manifests and guarded activation in the existing SurrealDB substrate | Persist native scientific versions through the established canonical owner |
| Scientific knowledge projections | runtime.modeling\_knowledge and runtime.modeling\_knowledge\_names expose admitted typed cells, uncertainty, provenance and test-only status | Extend derived discovery only for actual missing queries; the existing projection is not a populated global catalog |
| Research and fit publication | Immutable research datasets, complete selections, calibration lineage and qualified-fit promotion through ordinary source deltas | Reuse these owners for measured data and fitted coefficients |
| Schema generation | Registry-owned types and generated storage/codecs | Add required storage shapes through the existing schema authority |

**Database-backed scientific definitions fit the current model.** Authored scientific authority does not mean filesystem-only .pse files. A native unit or formulation definition can be stored as immutable typed declaration objects in SurrealDB, with the same admission semantics as source-authored definitions. Source text remains an authoring, review and interchange representation. Catalog metadata refers to those actual definitions and their revisions; it does not acquire separate editable equation authority.

Sources: [modeling ownership](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/models-and-composition.md), [typed source deltas](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/modeling/documents.rs), [contextual delta admission](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/modeling/canonical/delta.rs), [canonical source runtime](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/modeling/canonical.rs), [bounded source staging](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-operations/src/canonical_staging.rs), [knowledge schema](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-schema/src/catalog/modeling_knowledge.rs), [research workflows](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/research.rs), [research publication](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/research_publication.rs), [schema authority](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/schema-and-relations.md).

This is source inspection against pse-arrow revision **19e7b49a67df24c1bf73005a6186e145e930d53a**. It does not establish an implemented general importer, populated scientific catalog or new numerical qualification. Check the selected native revision and actual records before claiming case coverage.

## 3. What should be cataloged

The following are conceptual item kinds and relationships. Map them to existing declarations and owners first; this is not a prescription for twelve new tables or classes.

| Item kind | Essential meaning and relationships |
| --- | --- |
| Source and source artifact | Publisher/provider, publication or software identity, exact release/commit/content hash, original locator, declared scope, source role and applicable code/data access terms |
| Material, species and characterization | Stable subject identities, aliases, composition/charge knowledge, phase identity and empirical or structural characterization where needed |
| Scientific formulation | Exact equation/potential/correlation or model composition, variant, inputs/outputs, physical conventions and applicability claims |
| Parameterization or fitted group | Exact formulation compatibility, fitting context, common parameter identities, dependencies, uncertainty/covariance and original fit evidence |
| Scientific record | Subject-specific constants, pure-component or interaction parameters, measured/fitted/predicted role, physical quantities and source occurrence |
| Experimental or plant dataset | Versioned dataset/series/observation structure, conditions, units/bases, uncertainty, sampling/aggregation, calibration and original source coordinates |
| Property or reaction package selection | Selected formulations, states/phases, species, parameter records, reference conventions and complete dependency closure |
| Unit model definition | Ports, internal states, balance assumptions, geometry, constitutive closures, property/reaction demands and authored definition reference |
| Supported unit configuration | Selected physical options and dependencies, specification roles, optional storage/spatial behavior and the scope actually supported |
| Native implementation binding | Admitted declaration and source revision or exact provider implementation/data version, observation and derivative capabilities, failure/effect contract |
| Reference case and evidence | Exact case/configuration/input revision, reference implementation or measurements, comparison purpose, results, limitations and independence |
| Engineering assessment model | Cost or impact formulation, selected physical boundary, period/base date and factors, applicability and evidence |

Large numeric populations may remain typed tables or sealed payloads under existing canonical owners, with queryable identities, schemas, provenance and dependencies in SurrealDB. A dataset and its addressable records can be database items without making every scalar a separate mutable object. Unit and formulation items can be the existing admitted typed declarations themselves; derived catalog metadata indexes them. Source files remain a useful authoring/review format, while canonical scientific versions may live in the database.

Preserve relations such as **derived from**, **parameterizes**, **requires**, **implements**, **uses**, **compared against** and **supersedes** with exact version coordinates. Distinguish a source assertion from its interpreted scientific meaning and from a native implementation. Multiple software distributions may reproduce the same original fit; preserve all occurrences and their common ancestry.

Existing declarations already separate methods, parameterizations, subject records, ordered/symmetric pairs, joint-fit groups, variants, applicability and selections. Use those distinctions. A pair called “NRTL, ethanol/water” is insufficient: the catalog needs the exact temperature dependence and coefficient convention, both directional records where required, nonrandomness convention, joint-fit identity, source and applicability. This example illustrates the contract; it does not claim that this particular pair has been imported or qualified. [Property declarations](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/domain/models/properties.pse); [physical semantics](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/physical-semantics.md).

## 4. Native Rust pipeline and SurrealDB ownership

| Responsibility | Owner to reuse |
| --- | --- |
| Source selection, acquisition orchestration and import receipts | Native Rust workflows within pse-arrow; extend existing source/artifact operations only for demonstrated format needs |
| Scientific interpretation, quantities, formulations, parameter records and unit composition | Existing authored scientific types and packages, admitted through pse-authoring and pse-modeling |
| Import preview, contextual admission and publication | Existing pse-runtime source workflows |
| Durable source objects, staging, revisions and activation | Existing pse-operations canonical owners and SurrealDB instance |
| Storage schema and codecs | pse-schema registry and pse-codegen |
| Measurements, selections, fitting and results | Existing research, publication and evidence owners |
| Scientific search and capability summaries | Derived indexed products of exact native scientific revisions |
| Design policy, source priorities and review links | Notion |

### Import contract

A bounded scientific import request should identify the consuming requirement/case, source role, exact release/commit/hash and locators, selected record or path population, extraction/mapping version, intended native identities and required dependency closure. Retain the original artifact, interpretation and transformations so each admitted record remains explainable. If the original artifact format needs a new retained payload type, extend the existing source/artifact owner for that concrete need.

The path is: **case demand → existing native coverage → exact source selection → Rust extraction and reconciliation → typed declaration/data delta → ordinary admission → guarded source publication → derived catalog and case use**.

Whole original artifacts may be retained when useful for reproducibility, while only the selected scientific content is interpreted and admitted. The first adapter should support the actual source shape needed by the first missing record. A general extraction language, universal Python-model translator or full-library mirror is not a prerequisite.

Unadmitted or ambiguous content remains retained source/draft/artifact material with explicit limitations. It does not need a second accepted-knowledge application or executable authority.

### Publication and update behavior

Preserve the existing staged publication pattern. Acquisition, parsing, reconciliation and private admission occur outside the activation transaction. Stage immutable content in bounded operations, close the source manifest, then activate through the existing exact-parent and guard checks. Expose catalog coverage for the relevant published revision. A changed external source produces an explicit new native revision and an impact report; existing cases retain their selected scientific versions.

Use existing operation identity and retry semantics. Repeating an unchanged import should be scientifically idempotent; identical content must not silently create a new parameter interpretation. [Source staging](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-operations/src/canonical_staging.rs); [identity and publication](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/identity-and-publication.md).

### Performance placement

The integration benefit is concrete: one native type system, source lifecycle, dependency graph, scientific store and evidence path. Faster execution is a performance objective to measure on representative work, not an established comparison between programming languages or database products.

Use SurrealDB for durable scientific definitions/data, indexed discovery, exact selections and lineage. Resolve and load the selected scientific dependency closure during preparation; repeated property, residual and derivative evaluation uses prepared native objects in memory. Retain existing typed Arrow/IPC/table representations where they suit numerical payloads. The current deployment and canonical storage boundaries remain applicable.

Measure selected-import throughput/working set, indexed discovery behavior, cold preparation versus reuse, and database calls in numerical hot paths only where these answer a concrete implementation question. Avoid arbitrary latency targets or a separate database-comparison project. [Efficient architecture principles](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/efficient-architecture-heuristics.md); [current architecture](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/architecture-overview.md); [SurrealDB performance guidance](https://surrealdb.com/docs/learn/querying/performance/performance-best-practices).

## 5. Source policy: select by role and preserve original structure

Source choice depends on the question. Matching an IDAES case, implementing a standard formulation and evaluating prediction against measurements require different evidence.

| Source family | Useful role and structured acquisition route | Selection obligation |
| --- | --- | --- |
| Pinned IDAES core, examples and compatibility sources | Actual property configurations, unit builders, equations, tests, notebook data and helper modules | Capture the selected scientific configuration and dependency set, not only a class name or notebook title |
| IAPWS releases and supplements | Normative water/steam formulations, coefficients, conventions, domains and published check material | Keep exact release/variant and formulation-bound constants; verify any document transcription |
| NIST ThermoML Archive | Programmatic experimental data via XML/JSON and archive/search interfaces | Preserve paper, sample, series, point, conditions and uncertainty; representation checking does not establish critical scientific evaluation |
| CoolProp and teqp | Native fluid/model JSON and mixture/departure collections; exact-version reference evaluations | Retain model/data/backend identities, parameter dependencies, phase and datum choices |
| thermo/chemicals | Structured property correlations, constants and interaction datasets | Distinguish actual source records from runtime defaults or inferred values |
| Cantera | Native YAML phases, species, thermo/transport and reaction definitions, with supported source-format converters | Preserve mechanism composition, units, rate conventions and dependencies |
| WaterTAP and PrOMMiS | Specialized unit configurations, property/reaction packages, parameter files and engineering examples | Preserve actual operating assumptions and data scope; defaults are not universally recommended engineering values |
| Original papers, standards and characterized plant/vendor data | Missing formulation detail, measured behavior, equipment correlations or local calibration | Record exact equation/table/page or dataset locator, transformations, source terms and applicability |

Primary references: [IDAES package configuration](https://idaes-pse.readthedocs.io/en/2.13.0/explanations/components/property_package/general/generic_definition.html), [IAPWS-95 release](https://iapws.org/technical-guidance/release/IAPWS-95), [ThermoML Archive](https://trc.nist.gov/ThermoML/), [CoolProp fluid JSON](https://coolprop.org/coolprop/HighLevelAPI.html#adding-fluids), [teqp multifluid data](https://teqp.readthedocs.io/en/latest/models/multifluid.html), [thermo interaction records](https://thermo.readthedocs.io/thermo.interaction_parameters.html), [Cantera YAML](https://cantera.org/stable/yaml/index.html), [WaterTAP zero-order parameters](https://watertap.readthedocs.io/en/stable/technical_reference/unit_models/zero_order_unit_models/index.html), [PrOMMiS solvent extraction](https://prommis.readthedocs.io/en/latest/_autosummary/prommis.solvent_extraction.solvent_extraction.html).

Prefer direct parsing of authoritative structured artifacts over manual transcription or treating a sampled API output as an original parameter record. Use source-specific semantic extraction or controlled builder inspection for code-defined models. Retain opaque/partial code knowledge when it cannot be translated faithfully; do not label it as an executable native unit. Literature extraction should retain exact locators and make ambiguity explicit. An agent may propose interpretations, but its generated text is not a scientific source.

Retain software-license and dataset-access facts separately. Source discovery, declared manifest scope, actual retained bytes, accepted interpretation and native scientific coverage are different observations.

**Unit-model scope clarification:** the user deliberately deferred unit definitions to this process-domain exercise. Their absence from the earlier thermo-knowledge intake is not a defect to repair there. Define native physical composition and supported configurations first; use selected IDAES unit sources, tests and examples as scientific references for those definitions. The sources listed above are candidates selected by demand, not a commitment to compile their entire libraries.

## 6. Agent workflow for each scientific increment

### 1. Derive the scientific demand

Start from a committed engineering case or reusable capability with an actual consumer. Record materials, phases, state coordinates, quantities, operating conditions, reaction/transfer requirements and derivatives. For a unit, include physical structure, balance/geometry assumptions, specification roles, storage/spatial options and failure behavior.

### 2. Discover native coverage

Query admitted packages, scientific records, retained sources and research datasets in the native system. Explain which dependencies are already usable, available but unmapped, incomplete, conflicting, unsupported or unqualified for this use. Use these explanations within existing owners, without introducing a new global readiness state machine.

### 3. Select sources for their roles

Distinguish the matched IDAES reference, formulation authority, parameter source, experimental evidence and independent implementation check. Select exact variants/fits and explain alternatives. An improved physical model is a separate choice when baseline equivalence requires matched physics. Acquire only content needed by the selected demand and its complete dependencies.

### 4. Extract through a bounded Rust adapter

Record exact resolved source pins/hashes and original locators. Prefer native structured artifacts and explicit source-specific mappings. Retain source values before conversion and the adapter version. Code-defined science may require deliberate interpretation into native declarations; avoid treating a retained Python class as an implemented native model. Preserve partial or ambiguous content explicitly.

### 5. Reconcile scientific meaning

Resolve identities, quantities, units, mass/molar/amount bases, phase and reference states, energy conventions, directional interactions and fit dependencies. Retain transformation meaning. Keep absent values, explicit zeros, predictive rules and estimates distinct. Preserve uncertainty, calibration context and joint fits. Competing assertions remain available for an explicit selection.

### 6. Admit and publish native content

Construct ordinary typed scientific declarations/data against the selected parent revision. Reuse normal contextual admission, source preview and guarded publication. Measurements and fitted coefficients use the existing research/publication owners. Preserve source roles and test-only/oracle restrictions. A catalog entry records the exact native definition and source version.

### 7. Qualify the claimed use

Choose evidence appropriate to the claim: published values, matched IDAES outcomes, physical identities, conservation, derivatives, phase/root behavior and independent observations. Separate calibration from prediction evidence. Preserve failed, unavailable and unattempted outcomes. Retained content, admitted native science and qualified engineering use remain distinct.

### 8. Expose queries and update impact

Publish the derived views needed to discover definitions, compatible records, required capabilities, supported configurations and evidence. Bind answers to covered revisions. On a source or fit change, identify affected selections, units and cases before publishing a new native revision. Requalify the affected claims while preserving original scientific inputs for existing cases.

## 7. Scientific details the catalog must preserve

A common envelope should provide identity/version, scientific kind, owner, source coordinates, dependencies, native binding and evidence links. Family-specific schemas retain the actual scientific payload. Avoid an untyped key/value store that loses physical meaning.

Three concrete examples show why:

1. **Formulation-bound constants.** IAPWS-95 specifies the gas constant used to obtain its residual coefficients and requires that same value in the formulation. Replacing it with a newer general constant would change the scientific model. The release, coefficients and constants must be selected together. [IAPWS-95 §2](https://iapws.org/technical-guidance/release/IAPWS-95.download).
2. **Missing interactions.** The thermo interaction API distinguishes a real stored parameter from a fallback default. Extraction should preserve actual record presence and the fallback rule separately. A returned zero does not establish a fitted zero. [Interaction database API](https://thermo.readthedocs.io/thermo.interaction_parameters.html).
3. **Starter equipment parameters.** WaterTAP describes zero-order YAML defaults as starting values and advises users to supply their own when possible. Such records are useful catalog entries with a limited role; they do not establish validated design values. [WaterTAP data guidance](https://watertap.readthedocs.io/en/stable/technical_reference/unit_models/zero_order_unit_models/index.html).

For equipment, distinguish the reusable unit definition, selected supported configuration, plant/model occurrence, operating specification and result. A heat exchanger catalog entry should reveal its balance assumptions, selected heat-transfer/pressure-drop closures, property demands, geometry and specification freedoms. A solved-area case and a fixed-area case can use the same physical definition while making different demands. Costing and initialization remain linked scopes with their own meaning.

Across industries, the same approach accommodates empirical lumps, assays, electrolyte species, mineral phases, particle distributions, polymer structure, biological populations and batch histories. Preserve the exactness of each conservation or characterization claim. Broader representation never justifies presenting an empirical lump as complete elemental chemistry.

### Reusable unit structure and engineering templates

Use a small, growing set of reusable physical compositions. The following distinctions are roles in the existing authored model, not a new class hierarchy or a table per row:

| Role | Meaning |
| --- | --- |
| Physical structure | Material regions, stream paths, states, boundaries, junctions and exchange interfaces |
| Balance contribution | Typed transport, accumulation, reaction and exchange terms with explicit ownership and sign |
| Constitutive closure | A relation supplying heat transfer, hydraulics, kinetics, equilibrium, partitioning, efficiency or geometry-dependent behavior |
| Unit definition/template | A reusable composition exposing a familiar engineering interface, useful defaults and explicit assumptions |
| Selected configuration | Chosen properties, closures, phases, geometry assumptions, representation and supported physical options |
| Equipment occurrence and case | One flowsheet instance, its connections and physical parameters, plus operating/design specifications |
| Numerical realization | Preparation, discretization, initialization, decomposition and solution of the selected equations |

A broad equipment catalog can contain many named templates while sharing comparatively few underlying implementations. Keep meaningful names such as Pump, Turbine and HeatExchanger. Their public interfaces should expose engineering intent; users need not manually assemble every balance or exchange.

The existing HeatExchanger already composes two Heater children, a selectable temperature-driving-force definition and conservative thermal exchange. Extend that family with compatible coefficient/pressure-loss closures, geometry and explicit storage or distributed physics as selected cases require. Rating and design cases should change fixed/free specifications on the same physical definition. [Exchanger definition](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/process/models/heat-exchanger.pse).

The existing pressure-changing family shares its balance/work structure, with distinct hydraulic-pump and isentropic assumptions and Pump/Compressor/Turbine presets. Preserve their different demands: the inspected pump consumes dilute-liquid volume flow, while the isentropic form needs entropy-capable states and a reference state. Reuse does not mean every package supports every configuration. [Pressure-changing definitions](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/process/models/pressure-changer.pse).

Interacting-stream structures can later support selected extraction, membrane and other contactor families with different transfer, equilibrium, hydraulic and reaction closures. IDAES MSContactor is a useful reference for reusable stream balances and exchanges; it explicitly does not use ControlVolume objects and leaves some reaction-extent closure to the application. A control-volume superclass must not become compulsory for all native units. [IDAES MSContactor](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/mscontactor.html).

Composition must check physical basis/datum and subject compatibility, demanded properties/records/derivatives, closure completeness, conservative exchange, structural consistency and applicable evidence. Prefer typed alternatives and named scientific assumptions over unrestricted combinations of flags; a Boolean is suitable only where it has a clear local meaning.

New physics still requires new scientific definitions and evidence. Boiling, phase transfer, wall inertia, particle populations and distributed accumulation are not obtained merely by changing a label. A lumped versus spatially distributed formulation can change physical assumptions; refining the mesh of an already selected distributed model is a numerical choice. Actual trays or stages retain physical identity separately from numerical cells. Current inventory and distributed-temporal limitations stay explicit until their closures are supplied.

## 8. Smallest useful delivery and roadmap integration

Keep the adopted P0–P7 sequence and first A–D packets. Add focused native sourcing and composition detail to their existing responsibilities.

| Existing step | Proposed additional deliverable |
| --- | --- |
| P0 scope/dependency inventory | List the exact formulations, records, unit configurations and source roles required by the first cases; distinguish existing content from actual missing inputs |
| Packet A ordinary scientific inputs | Demonstrate selected source → Rust extraction → admitted native record → SurrealDB publication → consuming case, preserving oracle/test-only eligibility |
| P2 properties/reactions | Make new formulations, fits, records, package selections and evidence relationships queryable through native owners |
| P3 equipment | Expose reusable unit definitions, typed closure requirements and supported configurations, with engineering-facing templates |
| P4–P6 later consumers | Add adapters or scientific structures when a selected inventory, distributed, specialized or assessment consumer proves the need |
| P7 scope closure | Close the finite supported asset/configuration/case population with evidence; imported row counts are a separate measure |

### First complete implementation example

1. **Inventory the existing cases.** Keep the benzene/toluene ideal recycle closure distinct from the exchanger-costing case's BT\_PR/relational-density closure. Identify the original sources and ordinary-input eligibility of each required record. Do not import a broader thermo library merely because it is available.
2. **Complete one native import path.** Reuse current typing, source deltas, staging, guarded activation and knowledge views. Add only the adapter and query products actually missing. Reopening the case must recover exact scientific inputs; repeating an unchanged import must preserve scientific identity.
3. **Demonstrate exchanger composition.** Use the same physical definition for area-specified rating and outlet-specified design. Demonstrate a compatible closure substitution without rewriting side balances; any changed science has its own evidence scope.
4. **Exercise flowsheet reuse.** Use the recycle case to test composition, connection, phase-aware transport and native initialization with the selected content.
5. **Choose the next distinct demand.** Let a committed dynamic-inventory or inter-stream-contact case expose the next missing structure. Do not build both families speculatively.

[Recycle case](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/campaign/models/recycle-flash.pse); [BT\_PR exchanger case](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/campaign/models/exchanger-costing.pse). The existing [NIST nitrogen Shomate declaration](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/data/nist/models/parameters.pse) is an optional small provenance/discovery example; it is not a dependency of these benzene/toluene process cases.

Acceptance should establish a source-to-case trace, discoverable definitions/dependencies, explicit missing-data explanations, reusable equipment composition and the original scoped scientific checks. Measure import/query/preparation behavior only as needed to resolve a concrete performance risk. Broad scientific content grows through subsequent cases while the existing finite IDAES equivalence population remains the target.

## 9. Acceptance questions and bounded coding brief

A useful catalog should answer these questions with exact item/revision links and explicit coverage limits:

- Which formulations and parameterizations can supply the demanded properties for these materials, phases and conditions?
- Which required interaction, transport, reaction or characterization records are absent, and which are merely not yet mapped?
- Which unit configurations can consume those capabilities, and what assumptions and additional closures do they require?
- Which original sources, transformations and native records did this case use?
- Why is a record eligible as an ordinary input, and what evidence supports this particular use?
- Which parameters were jointly fitted, from which observations, and which evidence is independent?
- Which model selections, units, cases and claims are affected by a source or fit revision?
- What is discoverable only as descriptive/partial knowledge, and what is actually admitted and executable?

**Proposed coding-agent brief:** Implement targeted scientific sourcing and catalog capabilities within pse-arrow's Rust codebase and existing SurrealDB instance. Begin with the selected case's complete scientific dependencies and existing native coverage. Produce ordinary typed declaration/data deltas with exact source, variant, fit, unit/basis/datum and applicability provenance. Reuse existing admission, staging, publication, research and evidence owners; generate any required schema through the current registry. Define equipment through reusable physical compositions, typed closures and meaningful engineering templates. Demonstrate a complete source-to-case path and rating/design reuse before expanding scientific coverage. Keep repeated numerical evaluation on prepared native data. Add a generic mechanism only when the consumer demonstrates the missing contract.

The completion evidence for this proposal is a reproducible, queryable source-to-case path and a truthful explanation of missing or unsupported content. This review itself supplies design guidance only; it does not establish a populated catalog or new numerical qualification.
