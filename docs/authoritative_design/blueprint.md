---
status: current
revision: 57
date: 2026-09-26
---

# Architecture blueprint: revisions and former anchors

The authoritative architecture is the set of focused pages under
[`sections/`](README.md), starting with the [reading guide](sections/reading-guide.md) and
the [architecture overview](sections/architecture-overview.md). Each numbered section has
exactly one owner there; section numbers are stable citation identities, so
`blueprint §14.3` means the §14.3 heading wherever it lives.

This file keeps two things: the collection's revision history, and a compatibility table
that maps every heading of the former single-file blueprint to its current owner. It holds
no normative text.

## Revision history

The decision route (`.claude/rules/decisions.md`) adds a row here when an ADR amends the
collection. Revisions 1–55 and the former single-file text are in Git history, for example
[`blueprint.md` at revision 55](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/authoritative_design/blueprint.md).

| Revision | Date | Change | git |
|---|---|---|---|
| 56 | 2026-09-26 | ADR-0096 / Plan 19: the mixed legacy blueprint is replaced by focused current contracts under `sections/`; retired mechanisms keep one-line identity pointers; former anchors map below. Product contracts unchanged. | ADR-0096 accepted by the maintainer |
| 57 | 2026-09-26 | ADR-0088–0095 accepted as implemented (Plan 16 contracts in §0.5–§0.6, §5, §9, §13, §18, §20, §24; §0.1, §0.3, §24.4); ADR-0095 supersedes the retired ADR-0033/0036. No contract text changed. | maintainer acceptance |

## Former anchors

Links into the former single-file blueprint land on a row below; follow it to the current
owner. A heading whose mechanism was retired leads to its one-line retirement pointer.

| Former blueprint heading | Current owner |
|---|---|
| <a id="0-purpose-scope-and-how-to-read-this-document"></a>§0 Purpose, scope, and how to read this document | [architecture overview: Purpose and scope](sections/architecture-overview.md#section-0) |
| <a id="02-what-core-idaes-pse-capabilities-means-here"></a>§0.2 What “core IDAES-PSE capabilities” means here | [architecture overview: What "core IDAES-PSE capabilities" means here](sections/architecture-overview.md#section-0-2) |
| <a id="05-current-native-process-simulator-contract"></a>§0.5 Current native process-simulator contract | [architecture overview: Current native process-simulator contract](sections/architecture-overview.md#section-0-5) |
| <a id="06-selected-data-model-foundation-amendments"></a>§0.6 Selected data-model foundation amendments | [architecture overview: Selected data-model foundation](sections/architecture-overview.md#section-0-6) |
| <a id="1-architectural-summary"></a>§1 Architectural summary | [architecture overview: Architectural summary](sections/architecture-overview.md#section-1) |
| <a id="11-the-one-paragraph-design"></a>§1.1 The one-paragraph design | [architecture overview: The design in brief](sections/architecture-overview.md#section-1-1) |
| <a id="12-the-three-representations-made-concrete"></a>§1.2 The three representations, made concrete | [architecture overview: The three representations, made concrete](sections/architecture-overview.md#section-1-2) |
| <a id="13-where-each-library-sits"></a>§1.3 Where each library sits | [architecture overview: Where each library sits](sections/architecture-overview.md#section-1-3) |
| <a id="2-architectural-decisions"></a>§2 Architectural decisions | [architecture overview: Architectural decisions](sections/architecture-overview.md#section-2) |
| <a id="d1-typed-relations-are-the-only-authority"></a>§D1 Typed relations are the only authority | [architecture overview: The registry declares each meaning once; authored definitions are the model](sections/architecture-overview.md#section-d1) |
| <a id="d2-author-causes-derive-consequences"></a>§D2 Author causes, derive consequences | [architecture overview: Author definitions and bindings; derive specialized consequences](sections/architecture-overview.md#section-d2) |
| <a id="d3-three-artifact-levels-one-schema-system"></a>§D3 Three artifact levels, one schema system | [architecture overview: Definitions, prepared products and attempts are distinct artifacts](sections/architecture-overview.md#section-d3) |
| <a id="d4-semantic-ids-artifact-local-ordinals-and-content-hashes-are-distinct"></a>§D4 Semantic IDs, artifact-local ordinals, and content hashes are distinct | [architecture overview: Semantic identities, projections, ordinals and storage versions are distinct](sections/architecture-overview.md#section-d4) |
| <a id="d5-physical-type-is-more-than-a-unit-string"></a>§D5 Physical type is more than a unit string | [architecture overview: Physical type is more than a unit string](sections/architecture-overview.md#section-d5) |
| <a id="d6-the-math-ir-is-richer-than-datafusion-expr"></a>§D6 The math IR is richer than DataFusion Expr | [architecture overview: Mathematics is library-owned and derived from typed definitions](sections/architecture-overview.md#section-d6) |
| <a id="d7-laws-are-templates-over-contributions"></a>§D7 Laws are templates over contributions | [architecture overview: Balances are generated from declared contributions](sections/architecture-overview.md#section-d7) |
| <a id="d8-property-demand-is-resolved-explicitly"></a>§D8 Property demand is resolved explicitly | [architecture overview: Property and reaction demand is bound explicitly](sections/architecture-overview.md#section-d8) |
| <a id="d9-kernels-have-one-contract-and-generated-adapters"></a>§D9 Kernels have one contract and generated adapters | [architecture overview: Physical providers have one typed contract](sections/architecture-overview.md#section-d9) |
| <a id="d10-choose-computation-per-operation-and-preserve-relational-authority"></a>§D10 Choose computation per operation and preserve relational authority | [architecture overview: Computation placement follows the operation](sections/architecture-overview.md#section-d10) |
| <a id="d11-native-numerics-own-execution-layouts"></a>§D11 Native numerics own execution layouts | [architecture overview: Immutable preparation, attempt-owned native execution](sections/architecture-overview.md#section-d11) |
| <a id="d12-native-execution-with-a-coarse-grained-python-boundary"></a>§D12 Native execution with a coarse-grained Python boundary | [architecture overview: Native class-specific execution; Python is the authoring and result boundary](sections/architecture-overview.md#section-d12) |
| <a id="d13-cases-and-results-never-mutate-the-model"></a>§D13 Cases and results never mutate the model | [architecture overview: Cases and results never mutate the model](sections/architecture-overview.md#section-d13) |
| <a id="d14-incrementality-follows-declared-dependencies"></a>§D14 Incrementality follows declared dependencies | [architecture overview: Reuse follows complete declared dependencies](sections/architecture-overview.md#section-d14) |
| <a id="21-doctrine-crosswalk"></a>§2.1 Doctrine crosswalk | [architecture overview: Doctrine crosswalk — retired](sections/architecture-overview.md#section-2-1) |
| <a id="3-workspace-crates-and-dependency-pins"></a>§3 Workspace, crates, and dependency pins | [workspace and dependencies: Workspace, crates and library boundaries](sections/workspace-and-dependencies.md#section-3) |
| <a id="31-version-anchors"></a>§3.1 Version anchors | [workspace and dependencies: Version authority and pinned families](sections/workspace-and-dependencies.md#section-3-1) |
| <a id="32-cargo-workspace-layout"></a>§3.2 Cargo workspace layout | [workspace and dependencies: Workspace crates and dependency direction](sections/workspace-and-dependencies.md#section-3-2) |
| <a id="33-supporting-libraries-and-the-boundary-each-must-respect"></a>§3.3 Supporting libraries and the boundary each must respect | [workspace and dependencies: Supporting libraries and their boundaries](sections/workspace-and-dependencies.md#section-3-3) |
| <a id="331-full-arrow-and-datafusion-capability-access"></a>§3.3.1 Full Arrow and DataFusion capability access | [workspace and dependencies: Arrow and DataFusion roles and capability eligibility](sections/workspace-and-dependencies.md#section-3-3-1) |
| <a id="332-dependency-admission-and-licence-policy"></a>§3.3.2 Dependency admission and licence policy | [workspace and dependencies: Dependency admission and licence policy](sections/workspace-and-dependencies.md#section-3-3-2) |
| <a id="333-computation-placement-and-hard-pivot-delivery"></a>§3.3.3 Computation placement and hard-pivot delivery | [workspace and dependencies: Computation placement](sections/workspace-and-dependencies.md#section-3-3-3) |
| <a id="4-the-semantic-schema-registry"></a>§4 The semantic schema registry | [schema and relations: The semantic schema registry](sections/schema-and-relations.md#section-4) |
| <a id="41-what-is-declared-once"></a>§4.1 What is declared once | [schema and relations: What is declared once](sections/schema-and-relations.md#section-4-1) |
| <a id="42-what-is-generated-from-it"></a>§4.2 What is generated from it | [schema and relations: What is generated from it](sections/schema-and-relations.md#section-4-2) |
| <a id="43-metadata-conventions"></a>§4.3 Metadata conventions | [schema and relations: Metadata conventions](sections/schema-and-relations.md#section-4-3) |
| <a id="44-extension-types"></a>§4.4 Extension types | [schema and relations: Extension types](sections/schema-and-relations.md#section-4-4) |
| <a id="45-logical-type-catalog-physical-scalars"></a>§4.5 Logical type catalog (physical scalars) | [schema and relations: Logical scalar types](sections/schema-and-relations.md#section-4-5) |
| <a id="46-construction-properties-and-residual-obligations"></a>§4.6 Construction properties and residual obligations | [schema and relations: Construction properties and residual obligations](sections/schema-and-relations.md#section-4-6) |
| <a id="5-identity-versions-snapshots-and-the-catalog"></a>§5 Identity, versions, snapshots, and the catalog | [identity and publication: Identity, revisions and exact selection](sections/identity-and-publication.md#section-5) |
| <a id="51-three-forms-of-identity"></a>§5.1 Three forms of identity | [identity and publication: Forms of identity](sections/identity-and-publication.md#section-5-1) |
| <a id="52-model-revision-case-revision-run"></a>§5.2 Model revision, case revision, run | [identity and publication: Model revisions, cases, runs and attempts](sections/identity-and-publication.md#section-5-2) |
| <a id="53-canonical-serialization-and-hashing"></a>§5.3 Canonical serialization and hashing | [identity and publication: Canonical framing and hashing](sections/identity-and-publication.md#section-5-3) |
| <a id="54-the-snapshot-catalog"></a>§5.4 The snapshot catalog | [identity and publication: Exact relation selection and the provider catalog](sections/identity-and-publication.md#section-5-4) |
| <a id="6-the-canonical-relation-families"></a>§6 The canonical relation families | [schema and relations: Relation families](sections/schema-and-relations.md#section-6) |
| <a id="61-identity-and-packages-authored-reference"></a>§6.1 Identity and packages (authored, reference) | [schema and relations: Identity, packages and source edits](sections/schema-and-relations.md#section-6-1) |
| <a id="62-physical-types-reference-extendable-by-packages"></a>§6.2 Physical types (reference, extendable by packages) | [schema and relations: Physical types](sections/schema-and-relations.md#section-6-2) |
| <a id="63-domains-index-sets-coordinates"></a>§6.3 Domains, index sets, coordinates | [schema and relations: Domains and index sets](sections/schema-and-relations.md#section-6-3) |
| <a id="64-material-systems-authored-elements-are-reference"></a>§6.4 Material systems (authored; elements are reference) | [schema and relations: Material systems and reactions](sections/schema-and-relations.md#section-6-4) |
| <a id="65-property-capability-registry-reference-extended-by-packages"></a>§6.5 Property capability registry (reference, extended by packages) | [schema and relations: Property and method declarations](sections/schema-and-relations.md#section-6-5) |
| <a id="66-templates-authored-reference-packages-ship-the-standard-library"></a>§6.6 Templates (authored; reference packages ship the standard library) | [schema and relations: Templates](sections/schema-and-relations.md#section-6-6) |
| <a id="67-instances-flowsheets-connectivity"></a>§6.7 Instances, flowsheets, connectivity | [schema and relations: Instances, compositions and connectivity](sections/schema-and-relations.md#section-6-7) |
| <a id="68-symbols-compiled"></a>§6.8 Symbols (compiled) | [schema and relations: Symbol declarations and roles](sections/schema-and-relations.md#section-6-8) |
| <a id="69-mathematics-compiledmath_"></a>§6.9 Mathematics (compiled.math_*) | [schema and relations: Compiled mathematics relations — retired](sections/schema-and-relations.md#section-6-9) |
| <a id="610-cases-specifications-observations-authored"></a>§6.10 Cases, specifications, observations (authored) | [schema and relations: Cases, observations, dynamics and fitting](sections/schema-and-relations.md#section-6-10) |
| <a id="611-numerical-infrastructure-reference-authored-compiled"></a>§6.11 Numerical infrastructure (reference, authored, compiled) | [schema and relations: Numerical requirements and native algorithm signatures](sections/schema-and-relations.md#section-6-11) |
| <a id="612-derived-structure-compiled"></a>§6.12 Derived structure (compiled) | [schema and relations: Derived structure relations — retired](sections/schema-and-relations.md#section-6-12) |
| <a id="613-execution-and-evidence-runtime-provenance"></a>§6.13 Execution and evidence (runtime, provenance) | [schema and relations: Execution results, publication and evidence](sections/schema-and-relations.md#section-6-13) |
| <a id="614-enumerations-preserved-from-idaes"></a>§6.14 Enumerations preserved from IDAES | [schema and relations: Enumerations preserved from IDAES](sections/schema-and-relations.md#section-6-14) |
| <a id="615-semantic-compilation-contracts"></a>§6.15 Semantic compilation contracts | [schema and relations: Semantic specialization contracts](sections/schema-and-relations.md#section-6-15) |
| <a id="6151-typed-configuration-and-finite-prospective-scopes"></a>§6.15.1 Typed configuration and finite prospective scopes | [schema and relations: Typed configuration and finite scopes](sections/schema-and-relations.md#section-6-15-1) |
| <a id="6152-rule-workspace-truth-and-support"></a>§6.15.2 Rule workspace, truth and support | [schema and relations: Rule workspace, truth and support — retired](sections/schema-and-relations.md#section-6-15-2) |
| <a id="6153-demand-and-method-selection"></a>§6.15.3 Demand and method selection | [schema and relations: Property demand and method selection](sections/schema-and-relations.md#section-6-15-3) |
| <a id="6154-instance-equations-and-indexed-realization"></a>§6.15.4 Instance equations and indexed realization | [schema and relations: Indexed realization of template declarations](sections/schema-and-relations.md#section-6-15-4) |
| <a id="6155-connections-topology-and-production-graph-context"></a>§6.15.5 Connections, topology and production graph context | [schema and relations: Connections and flowsheet topology](sections/schema-and-relations.md#section-6-15-5) |
| <a id="6156-pass-ownership-and-publication"></a>§6.15.6 Pass ownership and publication | [schema and relations: Pass ownership and publication — retired](sections/schema-and-relations.md#section-6-15-6) |
| <a id="6157-shipped-package-and-generated-leaf-fixture"></a>§6.15.7 Shipped package and generated leaf fixture | [schema and relations: Shipped reference packages and generated physical fixtures](sections/schema-and-relations.md#section-6-15-7) |
| <a id="7-the-mathematical-ir"></a>§7 The mathematical IR | [mathematics and compilation: Library-owned process mathematics](sections/mathematics-and-compilation.md#section-7) |
| <a id="71-design-constraints"></a>§7.1 Design constraints | [mathematics and compilation: Design constraints](sections/mathematics-and-compilation.md#section-7-1) |
| <a id="72-operator-catalog"></a>§7.2 Operator catalog | [mathematics and compilation: Function vocabulary and admission](sections/mathematics-and-compilation.md#section-7-2) |
| <a id="73-operator-contract-record"></a>§7.3 Operator contract record | [mathematics and compilation: Operation contracts: physical admission and domain obligations](sections/mathematics-and-compilation.md#section-7-3) |
| <a id="74-canonicalization-pass-p10"></a>§7.4 Canonicalization (pass P10) | [mathematics and compilation: Normalization and exact literals](sections/mathematics-and-compilation.md#section-7-4) |
| <a id="75-equation-records"></a>§7.5 Equation records | [mathematics and compilation: Rows, contributions and established facts](sections/mathematics-and-compilation.md#section-7-5) |
| <a id="76-null-bound-and-unknown-semantics"></a>§7.6 Null, bound, and unknown semantics | [mathematics and compilation: Null, bound, and unknown semantics](sections/mathematics-and-compilation.md#section-7-6) |
| <a id="77-the-expression-dsl"></a>§7.7 The expression DSL | [mathematics and compilation: The expression DSL](sections/mathematics-and-compilation.md#section-7-7) |
| <a id="8-physical-typing"></a>§8 Physical typing | [physical semantics: Physical typing](sections/physical-semantics.md#section-8) |
| <a id="81-quantitytype"></a>§8.1 QuantityType | [physical semantics: QuantityType](sections/physical-semantics.md#section-8-1) |
| <a id="82-units-and-unit-sets"></a>§8.2 Units and unit sets | [physical semantics: Units and unit sets](sections/physical-semantics.md#section-8-2) |
| <a id="83-unit-inference-algorithm-p10-step-4"></a>§8.3 Unit inference algorithm (P10 step 4) | [physical semantics: Quantity inference](sections/physical-semantics.md#section-8-3) |
| <a id="84-bases-and-reference-states"></a>§8.4 Bases and reference states | [physical semantics: Bases and reference states](sections/physical-semantics.md#section-8-4) |
| <a id="85-nominal-magnitudes"></a>§8.5 Nominal magnitudes | [physical semantics: Nominal magnitudes](sections/physical-semantics.md#section-8-5) |
| <a id="9-material-systems-and-the-property-framework"></a>§9 Material systems and the property framework | [physical semantics: Materials, properties and reactions](sections/physical-semantics.md#section-9) |
| <a id="91-from-genericparameterblock-configuration-to-relations"></a>§9.1 From GenericParameterBlock configuration to relations | [physical semantics: Material declarations](sections/physical-semantics.md#section-9-1) |
| <a id="92-state-definitions-are-templates"></a>§9.2 State definitions are templates | [physical semantics: State coordinates](sections/physical-semantics.md#section-9-2) |
| <a id="93-the-method-registry"></a>§9.3 The method registry | [physical semantics: Property method data](sections/physical-semantics.md#section-9-3) |
| <a id="94-the-equation-of-state-contract"></a>§9.4 The equation-of-state contract | [physical semantics: Physical provider contract](sections/physical-semantics.md#section-9-4) |
| <a id="95-phase-equilibrium"></a>§9.5 Phase equilibrium | [physical semantics: Phase equilibrium](sections/physical-semantics.md#section-9-5) |
| <a id="96-property-demand-resolution-pass-p6"></a>§9.6 Property demand resolution (pass P6) | [physical semantics: Property demand](sections/physical-semantics.md#section-9-6) |
| <a id="97-reaction-packages"></a>§9.7 Reaction packages | [physical semantics: Reaction binding](sections/physical-semantics.md#section-9-7) |
| <a id="98-provider-kernels-helmholtz-coolprop-feos"></a>§9.8 Provider kernels: Helmholtz, CoolProp, FeOs | [physical semantics: Provider implementations](sections/physical-semantics.md#section-9-8) |
| <a id="99-electrolytes-and-inherent-reactions"></a>§9.9 Electrolytes and inherent reactions | [physical semantics: Electrolytes and inherent reactions](sections/physical-semantics.md#section-9-9) |
| <a id="910-scaling-defaults-and-validity"></a>§9.10 Scaling defaults and validity | [physical semantics: Scaling defaults and validity](sections/physical-semantics.md#section-9-10) |
| <a id="10-balance-laws-and-control-volumes"></a>§10 Balance laws and control volumes | [models and composition: Conservation laws and control volumes](sections/models-and-composition.md#section-10) |
| <a id="101-the-conservation-law-template"></a>§10.1 The conservation law template | [models and composition: Law instances over contributions](sections/models-and-composition.md#section-10-1) |
| <a id="102-contribution-kinds"></a>§10.2 Contribution kinds | [models and composition: Contribution roles and signs](sections/models-and-composition.md#section-10-2) |
| <a id="103-the-lumped-control-volume-template-cvlumped1-idaes-controlvolume0dblock"></a>§10.3 The lumped control volume template (cv.lumped@1, IDAES ControlVolume0DBlock) | [models and composition: Lumped control volumes, steady and dynamic](sections/models-and-composition.md#section-10-3) |
| <a id="104-the-distributed-control-volume-template-cvdistributed_1d1-idaes-controlvolume1dblock"></a>§10.4 The distributed control volume template (cv.distributed_1d@1, IDAES ControlVolume1DBlock) | [models and composition: Distributed control volume — retired](sections/models-and-composition.md#section-10-4) |
| <a id="105-what-the-template-model-changes-structurally"></a>§10.5 What the template model changes structurally | [models and composition: Formulation selection without code](sections/models-and-composition.md#section-10-5) |
| <a id="11-the-unit-model-library-as-templates"></a>§11 The unit model library as templates | [models and composition: Unit models](sections/models-and-composition.md#section-11) |
| <a id="111-catalog"></a>§11.1 Catalog | [models and composition: What ships and what executes](sections/models-and-composition.md#section-11-1) |
| <a id="112-worked-template-the-heater"></a>§11.2 Worked template: the heater | [models and composition: Worked variant: adding heat input to a unit](sections/models-and-composition.md#section-11-2) |
| <a id="113-worked-template-fragment-isentropic-pressure-change"></a>§11.3 Worked template fragment: isentropic pressure change | [models and composition: Isentropic pressure-change fragment — retired](sections/models-and-composition.md#section-11-3) |
| <a id="114-derived-templates"></a>§11.4 Derived templates | [models and composition: Variants and narrowing](sections/models-and-composition.md#section-11-4) |
| <a id="12-connectivity"></a>§12 Connectivity | [models and composition: Connectivity](sections/models-and-composition.md#section-12) |
| <a id="121-ports"></a>§12.1 Ports | [models and composition: Typed ports](sections/models-and-composition.md#section-12-1) |
| <a id="122-connection-rules"></a>§12.2 Connection rules | [models and composition: Connection admission](sections/models-and-composition.md#section-12-2) |
| <a id="123-arc-expansion-is-a-pass-not-a-transformation-call"></a>§12.3 Arc expansion is a pass, not a transformation call | [models and composition: Connection expansion during selected lowering](sections/models-and-composition.md#section-12-3) |
| <a id="124-value-propagation-is-a-case-operation"></a>§12.4 Value propagation is a case operation | [models and composition: Values across connections](sections/models-and-composition.md#section-12-4) |
| <a id="125-topology-closure"></a>§12.5 Topology closure | [models and composition: Flow projection and recycle structure](sections/models-and-composition.md#section-12-5) |
| <a id="126-translators-state-junctions-and-scalers"></a>§12.6 Translators, state junctions, and scalers | [models and composition: Translation between conventions](sections/models-and-composition.md#section-12-6) |
| <a id="13-flowsheets-time-and-dynamics"></a>§13 Flowsheets, time, and dynamics | [workflows and results: Flowsheets, time and dynamics](sections/workflows-and-results.md#section-13) |
| <a id="131-the-flowsheet-template-and-time-domain"></a>§13.1 The flowsheet template and time domain | [workflows and results: Time domain and origins](sections/workflows-and-results.md#section-13-1) |
| <a id="132-dynamic-and-holdup-inference"></a>§13.2 Dynamic and holdup inference | [workflows and results: Declared dynamic roles](sections/workflows-and-results.md#section-13-2) |
| <a id="133-accumulation-terms-and-derivative-symbols"></a>§13.3 Accumulation terms and derivative symbols | [workflows and results: Time derivatives and accumulation](sections/workflows-and-results.md#section-13-3) |
| <a id="134-discretization-is-a-lowering-pass-p11"></a>§13.4 Discretization is a lowering pass (P11) | [workflows and results: Discretization lowering pass — retired](sections/workflows-and-results.md#section-13-4) |
| <a id="135-dynamic-operations-as-case-operations"></a>§13.5 Dynamic operations as case operations | [workflows and results: Dynamic operations over immutable revisions](sections/workflows-and-results.md#section-13-5) |
| <a id="136-trajectory-backends"></a>§13.6 Trajectory backends | [workflows and results: Native integrators and trajectories](sections/workflows-and-results.md#section-13-6) |
| <a id="14-the-compiler"></a>§14 The compiler | [mathematics and compilation: Compilation and preparation](sections/mathematics-and-compilation.md#section-14) |
| <a id="141-pass-pipeline-and-contracts"></a>§14.1 Pass pipeline and contracts | [mathematics and compilation: Preparation stages and contracts](sections/mathematics-and-compilation.md#section-14-1) |
| <a id="142-relational-invariants"></a>§14.2 Relational invariants | [mathematics and compilation: Relational invariants](sections/mathematics-and-compilation.md#section-14-2) |
| <a id="1421-optional-predicate-preimages"></a>§14.2.1 Optional predicate preimages | [mathematics and compilation: Optional predicate preimages — retired](sections/mathematics-and-compilation.md#section-14-2-1) |
| <a id="143-the-pass-engine"></a>§14.3 The pass engine | [mathematics and compilation: The preparation engine and ownership](sections/mathematics-and-compilation.md#section-14-3) |
| <a id="1431-bound-native-computation-and-completion"></a>§14.3.1 Bound native computation and completion | [mathematics and compilation: Artifact construction, retention and completion](sections/mathematics-and-compilation.md#section-14-3-1) |
| <a id="1432-consolidated-native-execution-and-ownership"></a>§14.3.2 Consolidated native execution and ownership | [mathematics and compilation: Resource ownership and admission limits](sections/mathematics-and-compilation.md#section-14-3-2) |
| <a id="144-incrementality"></a>§14.4 Incrementality | [mathematics and compilation: Incrementality](sections/mathematics-and-compilation.md#section-14-4) |
| <a id="145-closure-proofs"></a>§14.5 Closure proofs | [mathematics and compilation: Admission closure and refusal](sections/mathematics-and-compilation.md#section-14-5) |
| <a id="15-structural-analysis-and-diagnostics"></a>§15 Structural analysis and diagnostics | [numerical execution: Structural analysis and diagnostics](sections/numerical-execution.md#section-15) |
| <a id="151-thresholds-defaults-preserved-from-idaes"></a>§15.1 Thresholds (defaults preserved from IDAES) | [numerical execution: Thresholds (defaults preserved from IDAES) — retired](sections/numerical-execution.md#section-15-1) |
| <a id="152-structural-checks-no-numerical-values-required"></a>§15.2 Structural checks (no numerical values required) | [numerical execution: Structural checks without numerical values](sections/numerical-execution.md#section-15-2) |
| <a id="153-native-structural-algorithms-pse-structural"></a>§15.3 Native structural algorithms (pse-structural) | [numerical execution: Structural algorithms](sections/numerical-execution.md#section-15-3) |
| <a id="154-numerical-checks-require-values"></a>§15.4 Numerical checks (require values) | [numerical execution: Numerical checks that require values](sections/numerical-execution.md#section-15-4) |
| <a id="155-advanced-analyses"></a>§15.5 Advanced analyses | [numerical execution: Advanced analyses](sections/numerical-execution.md#section-15-5) |
| <a id="156-reports"></a>§15.6 Reports | [numerical execution: Reports](sections/numerical-execution.md#section-15-6) |
| <a id="16-scaling-as-an-explicit-transformation"></a>§16 Scaling as an explicit transformation | [numerical execution: Numerical policy and scaling](sections/numerical-execution.md#section-16) |
| <a id="161-model"></a>§16.1 Model | [numerical execution: Resolved numerical policy and normalization](sections/numerical-execution.md#section-16-1) |
| <a id="162-sources-of-scaling-factors-in-precedence-order"></a>§16.2 Sources of scaling factors, in precedence order | [numerical execution: Sources and precedence](sections/numerical-execution.md#section-16-2) |
| <a id="163-nominal-value-algebra-constraint-scaling-without-a-solved-point"></a>§16.3 Nominal value algebra (constraint scaling without a solved point) | [numerical execution: Nominal value algebra — retired](sections/numerical-execution.md#section-16-3) |
| <a id="164-scaler-templates"></a>§16.4 Scaler templates | [numerical execution: Scaler templates — retired](sections/numerical-execution.md#section-16-4) |
| <a id="165-profiling-and-persistence"></a>§16.5 Profiling and persistence | [numerical execution: Identity, provenance and persistence](sections/numerical-execution.md#section-16-5) |
| <a id="17-initialization-plans"></a>§17 Initialization plans | [numerical execution: Initialization, starts and recycles](sections/numerical-execution.md#section-17) |
| <a id="171-plan-model"></a>§17.1 Plan model | [numerical execution: Structural initialization model](sections/numerical-execution.md#section-17-1) |
| <a id="172-standard-plan-templates"></a>§17.2 Standard plan templates | [numerical execution: Standard plan templates — retired](sections/numerical-execution.md#section-17-2) |
| <a id="173-plug-ins-and-initialization-order"></a>§17.3 Plug-ins and initialization order | [numerical execution: Plug-ins and initialization order — retired](sections/numerical-execution.md#section-17-3) |
| <a id="174-flowsheet-and-dynamic-plans"></a>§17.4 Flowsheet and dynamic plans | [numerical execution: Flowsheet recycles and dynamic starts](sections/numerical-execution.md#section-17-4) |
| <a id="175-continuation-homotopy"></a>§17.5 Continuation (homotopy) | [numerical execution: Continuation](sections/numerical-execution.md#section-17-5) |
| <a id="18-backends"></a>§18 Backends | [numerical execution: Native class-specific execution](sections/numerical-execution.md#section-18) |
| <a id="181-the-canonicalmathproblem"></a>§18.1 The CanonicalMathProblem | [numerical execution: Problem representations](sections/numerical-execution.md#section-18-1) |
| <a id="182-the-native-evaluation-program"></a>§18.2 The native evaluation program | [numerical execution: Evaluation programs and callback boundary](sections/numerical-execution.md#section-18-2) |
| <a id="183-in-process-ipopt"></a>§18.3 In-process Ipopt | [numerical execution: In-process NLP: Ipopt C and POUNCE](sections/numerical-execution.md#section-18-3) |
| <a id="184-the-nl-backend"></a>§18.4 The NL backend | [numerical execution: NL backend — retired](sections/numerical-execution.md#section-18-4) |
| <a id="185-kernel-adapters-generated-from-kernelspec"></a>§18.5 Kernel adapters generated from KernelSpec | [numerical execution: Kernel adapters generated from `KernelSpec` — retired](sections/numerical-execution.md#section-18-5) |
| <a id="186-solver-neutral-results"></a>§18.6 Solver-neutral results | [numerical execution: Truthful outcomes](sections/numerical-execution.md#section-18-6) |
| <a id="187-solve-plan-selection"></a>§18.7 Solve-plan selection | [numerical execution: Capability, eligibility and selection](sections/numerical-execution.md#section-18-7) |
| <a id="188-threading"></a>§18.8 Threading | [numerical execution: Threading, cancellation and resource ownership](sections/numerical-execution.md#section-18-8) |
| <a id="189-backend-capability-matrix"></a>§18.9 Backend capability matrix | [numerical execution: Capability matrix](sections/numerical-execution.md#section-18-9) |
| <a id="19-cases-results-and-analytics"></a>§19 Cases, results, and analytics | [workflows and results: Cases, results and analytics](sections/workflows-and-results.md#section-19) |
| <a id="191-cases-and-overlays"></a>§19.1 Cases and overlays | [workflows and results: Cases and overlays](sections/workflows-and-results.md#section-19-1) |
| <a id="192-results-and-reporting"></a>§19.2 Results and reporting | [workflows and results: Results, qualification and diagnostics](sections/workflows-and-results.md#section-19-2) |
| <a id="193-sweeps-and-convergence-studies"></a>§19.3 Sweeps and convergence studies | [workflows and results: Sweeps and reuse](sections/workflows-and-results.md#section-19-3) |
| <a id="194-parameter-estimation-and-data-reconciliation"></a>§19.4 Parameter estimation and data reconciliation | [workflows and results: Parameter estimation](sections/workflows-and-results.md#section-19-4) |
| <a id="195-costing"></a>§19.5 Costing | [workflows and results: Costing](sections/workflows-and-results.md#section-19-5) |
| <a id="196-utility-minimization-durangrossmann"></a>§19.6 Utility minimization (Duran–Grossmann) | [workflows and results: Utility minimization](sections/workflows-and-results.md#section-19-6) |
| <a id="197-optionality"></a>§19.7 Optionality | [workflows and results: Optionality](sections/workflows-and-results.md#section-19-7) |
| <a id="198-uncertainty"></a>§19.8 Uncertainty | [workflows and results: Uncertainty](sections/workflows-and-results.md#section-19-8) |
| <a id="20-persistence-provenance-and-reproducibility"></a>§20 Persistence, provenance, and reproducibility | [identity and publication: Persistence, publication and reproducibility](sections/identity-and-publication.md#section-20) |
| <a id="201-delta-durable-relations"></a>§20.1 Delta durable relations | [identity and publication: Delta durable relations](sections/identity-and-publication.md#section-20-1) |
| <a id="202-coherent-publication"></a>§20.2 Coherent publication | [identity and publication: Exact coherent publication](sections/identity-and-publication.md#section-20-2) |
| <a id="203-what-a-run-references"></a>§20.3 What a run references | [identity and publication: What a run references](sections/identity-and-publication.md#section-20-3) |
| <a id="204-reproduction-reuse-and-retention"></a>§20.4 Reproduction, reuse and retention | [identity and publication: Reproduction, reuse and retention](sections/identity-and-publication.md#section-20-4) |
| <a id="205-current-contracts-and-schema-evolution"></a>§20.5 Current contracts and schema evolution | [identity and publication: Current contracts and schema evolution](sections/identity-and-publication.md#section-20-5) |
| <a id="21-the-python-boundary-and-the-pyomo-adapter"></a>§21 The Python boundary and the Pyomo adapter | [workflows and results: The Python boundary](sections/workflows-and-results.md#section-21) |
| <a id="211-extension-module"></a>§21.1 Extension module | [workflows and results: Extension module, jobs and Arrow streams](sections/workflows-and-results.md#section-21-1) |
| <a id="212-adapter-algorithm-python"></a>§21.2 Adapter algorithm (Python) | [workflows and results: Pyomo adapter algorithm — retired](sections/workflows-and-results.md#section-21-2) |
| <a id="213-uses-of-the-adapter"></a>§21.3 Uses of the adapter | [workflows and results: Uses of the Pyomo adapter — retired](sections/workflows-and-results.md#section-21-3) |
| <a id="214-opaque-kernels-and-asl"></a>§21.4 Opaque kernels and ASL | [workflows and results: Opaque kernels and ASL — retired](sections/workflows-and-results.md#section-21-4) |
| <a id="215-python-contracts"></a>§21.5 Python contracts | [workflows and results: Python contracts](sections/workflows-and-results.md#section-21-5) |
| <a id="216-the-array-boundary-numpy"></a>§21.6 The array boundary (numpy) | [workflows and results: The array boundary (numpy)](sections/workflows-and-results.md#section-21-6) |
| <a id="22-authoring-and-the-extension-model"></a>§22 Authoring and the extension model | [models and composition: Authoring and the extension model](sections/models-and-composition.md#section-22) |
| <a id="221-package-layout"></a>§22.1 Package layout | [models and composition: Package layout](sections/models-and-composition.md#section-22-1) |
| <a id="222-the-change-set-model"></a>§22.2 The change-set model | [models and composition: Revisions and edits](sections/models-and-composition.md#section-22-2) |
| <a id="223-what-is-data-and-what-is-code"></a>§22.3 What is data and what is code | [models and composition: What is data and what is code](sections/models-and-composition.md#section-22-3) |
| <a id="224-agent-change-sets"></a>§22.4 Agent change sets | [models and composition: Agent changes](sections/models-and-composition.md#section-22-4) |
| <a id="23-observability-and-failure-semantics"></a>§23 Observability and failure semantics | [operations and validation: Observability and failure semantics](sections/operations-and-validation.md#section-23) |
| <a id="231-observability"></a>§23.1 Observability | [operations and validation: Observability](sections/operations-and-validation.md#section-23-1) |
| <a id="232-failure-taxonomy"></a>§23.2 Failure taxonomy | [operations and validation: Failure taxonomy](sections/operations-and-validation.md#section-23-2) |
| <a id="24-testing-and-acceptance"></a>§24 Testing and acceptance | [operations and validation: Testing and qualification](sections/operations-and-validation.md#section-24) |
| <a id="241-test-layers"></a>§24.1 Test layers | [operations and validation: Test layers](sections/operations-and-validation.md#section-24-1) |
| <a id="242-vertical-slices-and-acceptance-criteria"></a>§24.2 Vertical slices and acceptance criteria | [operations and validation: Current qualification basis](sections/operations-and-validation.md#section-24-2) |
| <a id="243-benchmarks"></a>§24.3 Benchmarks | [operations and validation: Benchmarks](sections/operations-and-validation.md#section-24-3) |
| <a id="25-delivery-phases"></a>§25 Delivery phases | [scope and open design: Supported scope and recorded limits](sections/scope-and-open-design.md#section-25) |
| <a id="26-risks-and-open-decisions"></a>§26 Risks and open decisions | [scope and open design: Risks and unresolved design choices](sections/scope-and-open-design.md#section-26) |
| <a id="appendix-a-idaes-capability-coverage-matrix"></a>Appendix A. IDAES capability coverage matrix | [scope and open design: Capability coverage against IDAES](sections/scope-and-open-design.md#capability-coverage-against-idaes) |
| <a id="appendix-b-relation-index"></a>Appendix B. Relation index | [scope and open design: Relation index](sections/scope-and-open-design.md#relation-index) |
| <a id="appendix-c-glossary"></a>Appendix C. Glossary | [scope and open design: Glossary](sections/scope-and-open-design.md#glossary) |
| <a id="01-what-this-document-is"></a>§0.1 What this collection is | [reading guide: What this collection is](sections/reading-guide.md#section-0-1) |
| <a id="03-reading-guide"></a>§0.3 Reading guide | [reading guide: Reading guide](sections/reading-guide.md#section-0-3) |
| <a id="04-conventions-used-in-this-document"></a>§0.4 Conventions used in this collection | [reading guide: Conventions used in this collection](sections/reading-guide.md#section-0-4) |
| <a id="244-architecture-review-and-design-change-tracking"></a>§24.4 Architecture review and design-change tracking | [design change workflow: Architecture review and design-change tracking](sections/design-change-workflow.md#section-24-4) |
