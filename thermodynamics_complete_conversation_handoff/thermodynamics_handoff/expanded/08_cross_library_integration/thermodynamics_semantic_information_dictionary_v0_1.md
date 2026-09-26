# Thermodynamics semantic information dictionary

**Document:** THERMO-INFORMATION-006  
**Version:** 0.1  
**Date:** 25 September 2026  
**Project:** Process simulator  
**Stage:** Step 6, specify information structures at semantic depth  
**Status:** Proposed language-independent semantic baseline. Catalog and authored-example arithmetic checked; no implementation conformance, provider execution or independent R/E/V certification.  
**Predecessors:** THERMO-SCOPE-001, THERMO-DWSIM-002, THERMO-LIBRARIES-003, THERMO-FUNCTIONAL-004 and THERMO-PACKAGING-005, each v0.1.

## 0. Decision, scope and reading guide

The information model separates **what a material means, which description of that material is being used, what physical question is asked, which numerical work is performed, what the returned information establishes, and whether that result is current**. The coherent engineering property package remains intact. None of these distinctions requires a separate database, process, class or service.

This document elaborates all eighteen Step-5 information products into 72 concept sheets, 372 semantic content facets, 108 explicit relationship contracts and 28 cross-concept invariants. Fifteen authored semantic examples show how the records cooperate. Every one of the 94 requirements and 34 scenarios has a retained information route; the ten Step-5 semantic owners and P1/P2/P3 commitments are unchanged.

The concrete design choices are reference-qualified quantities; a local material account that prevents duplicated inventory; separate constituent and representation identity; snapshot-local phase entries with explicit cross-snapshot correspondence; formula-qualified parameters; coherent caloric/chemical conventions; original-problem and attempt separation; and acceptance/currentness as different relationships.

Read §§1–3 for interpretation and the overall information map; §§4–6 for concept sheets, cardinalities and global rules; §7 for concrete examples; §§8–10 for requirement/scenario trace and unresolved decisions. Detailed action preconditions, state transitions, sequencing and recovery procedures belong to Step 7. Physical serialization layouts, database normalization, Arrow/Rust types, ID encoding and library selection remain downstream decisions.

### Evidence and precedence

B1 controls scope, B4 controls required behavior, and B5 controls ownership. B2/B3 motivate the choices but retain their original research limitations and source pins. This stage transforms those fixed baselines; it does not refresh external library features, select a backend or assert new production capabilities. New concept names and example arithmetic are authored design, not claims that any listed library already implements this dictionary.

**Minimum-content cardinalities apply to a completed record in the stated role.** Drafts can preserve explicit unresolved assertions but cannot claim completed interpretation, readiness or acceptance when mandatory semantics are unknown. The ability to represent a limited empirical material does not remove P2 delivery obligations; the ability to reject a request does not establish numerical coverage.

## 1. Shared semantic rules

### 1.1 Identity, revision and currentness

A **lineage identity** refers to the thing being modeled across revisions. An **exact revision** fixes its definition. A **content fingerprint** can help detect changes but does not prove scientific equivalence. An **equivalence/compatibility assessment** explicitly states what is interchangeable and for which operation. A **runtime session ID** refers to a numerical resource. A **current binding** states which exact approved definition/result applies now at a process scope. None substitutes for another.

Historical scientific records retain exact references. A current-name alias must be resolved before capture; it is not a valid substitute for an old immutable dependency. Multiple sessions can realize one package revision; two equal-looking results from different conventions can still have different meaning. A local phase need not have a global persistent identity across coalescence; a correspondence record supplies the defensible relationship.

### 1.2 Information shapes, not physical schemas

A sheet describes semantic content, required relationships, allowed variants and invalid combinations. Its content facets may be realized as embedded values, referenced objects, relational facts or computed views later. A facet containing several related concepts denotes a semantic group; it is not a requirement to pack them into one field or create foreign keys. The explicit relationship catalog, not the contextual related-concept list, states directional multiplicities.

`1` means exactly one semantic group, `0..1` optional or conditional singular, `1..*` one or more, and `0..*` zero or more. Conditions identify when otherwise optional information becomes mandatory. Several referenced concepts in a group do not mean one instance of every type. Every facet gives its meaning and unit/basis obligations. Repeated instances of a related concept may have different named roles, such as source and target representation; those are separate relations.

Definitions, observations, assessments, proposals and runtime records have different lifetimes. Defining a quantity does not require storing its evaluated values. Producing a state in native code does not transfer ownership of material semantics to the provider. P10 commits current references while the named package remains owner of the content's meaning and invariants.

### 1.3 Value presence is not a number

Every quantity assertion separates **availability**, **role**, **basis/convention**, **source**, and **quality**. Availability may be a known value (including zero), a stated interval, unresolved input, deferred calculation, physically undefined quantity, unsupported property, failed calculation, or withheld output. An unaccepted candidate may still contain known numerical values; its acceptance and publication status are separate from value availability. These distinctions cannot be encoded solely as zero, an empty vector or NaN.

The role can be fixed specification, observation, guess, approved default, supplied allocation or calculated output. An observation can be retained as a check instead of an extra equation. A guess can be numerically identical to a fixed input and still have different authority. Missing uncertainty is not zero uncertainty; parameter covariance, measurement error, numerical tolerance and model discrepancy are different evidence.

### 1.4 Quantity interpretation

The common description identifies the observable, subject/account/domain/component or phase pair, scalar/vector/tensor shape and axes, dimensions, display units, normalization basis, denominator and necessary reference/standard-state conventions. Extensive totals require an actual or explicitly declared reference amount. A reference-amount calculation may support intensive results without asserting physical holdup. Transfer rates require their temporal basis; an inventory has no implicit seconds denominator.

Simple display-unit conversion is not the same as mass-to-molar conversion, standard-condition volume conversion, a chemical-species map or a reference-model transition. Each latter operation needs its additional data and convention. Pressure/temperature used for a report must be identified as actual or reference conditions. An overall phase fraction names both its amount basis and the portions included in its denominator.

### 1.5 Minimum mathematical invariants

For a declared nonreactive amount partition, `n_i = Σ_α n_(i,α)` over **disjoint physical portions only**. Aggregate and apparent/true aliases are not extra terms. For compatible nonnegative amount coordinates, normalized fractions are meaningful when the denominator is positive; a separately supplied zero-flow feed composition remains a specification, not a quotient of zeros.

For a justified linear conserved basis `b=A n`, a reaction matrix must satisfy `A ν=0` for each claimed closed conserved quantity. An open exchange uses `b_after=b_before+b_exchange` with the sign and time basis explicit. A mapping `n_t=M n_s` preserves a declared basis when `A_t M=A_s` on its stated domain. State-dependent chemical reconciliation need not be a constant linear map.

An energy reference transformation may have the declared form `H_b=H_a+Σ_i n_i c_i` for constant species molar offsets. Its specific/molar counterpart depends on composition. Nonreacting constituent balances can cancel these offsets; reactions may not. The selected chemical/energy formulation must reconcile the reference contributions consistently. The existence of an energy bookkeeping transform does not license arbitrary independent changes to chemical-potential standards or equilibrium constants.

For a normalized composition chart with `x_N=1−Σ_(i<N) x_i`, derivatives are chart-dependent. The derivative of a property with phase allocation frozen is different from its response after phase/chemical re-equilibration. A transformation requires the applicable chain rule, including all dependent coordinates and reference contributions. At a transition, a unique smooth derivative may be undefined or unassessed and must not be invented.

These are semantic algebraic constraints under explicitly declared models, not numerical algorithms or universal error tolerances. Exact physical unit/data/profile choices remain prerequisites to numerical validation.

### 1.6 What is allowed to remain opaque

A whole-provider package may identify an inseparable model/data bundle rather than exposing every coefficient. It must still identify the actual artifact/build, exported operation semantics, conventions known at the boundary and restrictions. Unknown internal choices are explicit claim limits. An opaque but attributable realization can be useful; an opaque attachment without quantity, transformation and persistence meaning cannot satisfy a required material representation.

### 1.7 Completeness, selection and dependency safety

A completed limited-property definition can have explicitly unsupported operations. It cannot conceal a required-but-missing coefficient as an intentional parameter-free model. Approval of estimated data is not independent scientific validation. Scientific validity, numerical readiness and initializer coverage remain separate assessments.

A calculation captures either an exact relevant dependency set or a conservative superseding context. Dependent physical, numerical and provider changes are assessed separately. If impact cannot be determined safely, conservative invalidation is correct. Reference values connecting semantic records may form cycles; this is not the Step-5 acyclic contract-construction graph or a requirement for recursive implementation imports. Revision ancestry, location inheritance, completion parentage and retry ancestry require their own acyclicity checks in an implementation.

## 2. Information map by owner

| Owner | Concepts in this dictionary | Primary concern |
| --- | --- | --- |
| P01 Material semantics and representations | IC-01, IC-02, IC-03, IC-04, IC-05, IC-06, IC-07, IC-08, IC-09 | Give every material description a defensible meaning before choosing a calculation method. |
| P02 Property evidence, parameters and characterization | IC-10, IC-11, IC-12, IC-13, IC-14, IC-15, IC-16, IC-17 | Produce attributable data and characterized-material proposals without silently redefining the model during evaluation. |
| P03 Thermodynamic methods and configured packages | IC-18, IC-19, IC-20, IC-21, IC-22, IC-23, IC-24 | Define a coherent, reusable engineering property package independently of where it is used and which live session executes it. |
| P04 Material states, quantities and local descriptions | IC-25, IC-26, IC-27, IC-28, IC-29, IC-30, IC-31, IC-32, IC-33, IC-71 | Represent material at a location without confusing it with its provider, solver workspace, flow connection or acceptance status. |
| P05 Thermodynamic problems and operation contracts | IC-34, IC-35, IC-36, IC-37, IC-38, IC-39, IC-40, IC-41, IC-42 | Define what each calculation means, what it may determine, and what it must establish, without choosing a particular numerical library or forcing separate phase and chemistry solvers. |
| P06 Chemistry definitions and participation | IC-43, IC-44, IC-45, IC-46, IC-47 | Define chemical transformation semantics and their property/energy needs independently of a reactor geometry and independently of where the numerical solve occurs. |
| P07 Flowsheet integration and use-case coordination | IC-48, IC-49, IC-50, IC-51, IC-52 | Connect material/property semantics to engineering actions without turning a property package into the process simulator. |
| P08 Provider realization and numerical execution | IC-53, IC-54, IC-55, IC-56, IC-57, IC-58, IC-72 | Realize declared operations through native methods or external engines while containing implementation-specific ordering, state and failures. |
| P09 Result qualification and coverage evidence | IC-59, IC-60, IC-61, IC-62, IC-63 | Determine what a result establishes, independently of whether a solver returned success and independently of which revision is currently displayed. |
| P10 Model revisions, publication and reconstruction | IC-64, IC-65, IC-66, IC-67, IC-68, IC-69, IC-70 | Keep approved definitions and current-result bindings coherent across edits, failed work, persistence and restoration. |

### A typical complete information chain

A material representation (IC-03) and fixed data (IC-14), optional chemistry (IC-45), methods and conventions form a resolved package (IC-20). A process location (IC-48) acquires an effective binding (IC-49). Local quantities and phases are captured in a state (IC-28) describing one material account (IC-25). The calculation problem (IC-35) fixes its constraints (IC-36), authority (IC-37), output meanings (IC-38/39/40) and checks (IC-41).

An attempt (IC-54) realizes that question through a qualified provider (IC-53), numerical policy (IC-72), optional initializer (IC-57), and scoped session (IC-55). Candidate evidence (IC-56) is assessed through check results (IC-59) and qualified result (IC-60). P10 adopts a matching current binding (IC-67) only after its publication decision (IC-66) checks exact revision/dependencies (IC-64) and live run authority (IC-70). The archive (IC-68) keeps semantic reconstruction information, not just final numbers or handles.

A coupled equation model uses the same meaning and authority through IC-40 rather than requiring repeated full flashes. A reporting view (IC-31) can add an evaluated quantity without re-owning its underlying material. Actual material/energy exchange (IC-71) is separate from a coordinate or reference transform.

## 3. Refinement of the eighteen Step-5 information products

### A01. Material and representation definition

**Semantic owner:** P01. **Primary concept:** [IC-03](#ic-03). **Decomposition:** [IC-03](#ic-03), [IC-01](#ic-01), [IC-02](#ic-02), [IC-04](#ic-04), [IC-05](#ic-05), [IC-06](#ic-06), [IC-08](#ic-08), [IC-09](#ic-09).

**Context:** An assessed material description before runtime binding; draft assertions may be unresolved.

**Minimum semantic content and multiplicity:** One representation revision; one or more constituent meanings and coordinate/basis/domain descriptions. Optional distribution/site definitions are required only when that representation uses them.

**Boundary:** Canonical material meaning, not a physical account or an EOS configuration.

**Consumers retained from B5:** P02, P03, P04, P05, P06, P07, P08, P09. **Current-model commitment:** P10, without transferring semantic ownership.

### A02. Representation map and reduction definition

**Semantic owner:** P01. **Primary concept:** [IC-07](#ic-07). **Decomposition:** [IC-07](#ic-07).

**Context:** A map definition with a declared source and target.

**Minimum semantic content and multiplicity:** Exactly one source and one target representation, a forward rule/constraint relation, conservation claims and explicit inverse/loss conditions.

**Boundary:** Applying the map creates a new mapped description, not a second inventory or an implied inverse.

**Consumers retained from B5:** P04, P06, P07, P08, P09. **Current-model commitment:** P10, without transferring semantic ownership.

### A03. Property evidence and resolved parameter snapshot

**Semantic owner:** P02. **Primary concept:** [IC-14](#ic-14). **Decomposition:** [IC-14](#ic-14), [IC-10](#ic-10), [IC-11](#ic-11), [IC-12](#ic-12), [IC-13](#ic-13), [IC-15](#ic-15).

**Context:** A fixed data selection for a declared use or trial.

**Minimum semantic content and multiplicity:** One snapshot; selected assertions/bundles or an explicit parameter-free determination; source/override lineage and unresolved prerequisites.

**Boundary:** Known-zero, missing, rule-derived, fitted and substituted values remain distinct.

**Consumers retained from B5:** P03, P06, P08, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A04. Characterization/fit proposal

**Semantic owner:** P02. **Primary concept:** [IC-17](#ic-17). **Decomposition:** [IC-17](#ic-17), [IC-16](#ic-16).

**Context:** A preparation campaign outcome, including a failed campaign.

**Minimum semantic content and multiplicity:** One captured recipe/input set; zero or more trials; one assessment; zero or one selected coordinated change proposal.

**Boundary:** Candidate identities/data remain proposals until accepted by their semantic owners and P10.

**Consumers retained from B5:** P01, P03, P07, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A05. Chemical definition and participation description

**Semantic owner:** P06. **Primary concept:** [IC-45](#ic-45). **Decomposition:** [IC-45](#ic-45), [IC-43](#ic-43), [IC-44](#ic-44), [IC-46](#ic-46), [IC-47](#ic-47).

**Context:** A reusable chemical-system definition.

**Minimum semantic content and multiplicity:** One chemical system with justified species/domains and conserved bases; optional enumerated reactions/laws, apparent/true reconciliation and reservoir requirements.

**Boundary:** One coupled phase/chemistry problem is permitted; a separate chemistry solver is not required.

**Consumers retained from B5:** P03, P05, P07, P08, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A06. Configured thermodynamic package revision

**Semantic owner:** P03. **Primary concept:** [IC-20](#ic-20). **Decomposition:** [IC-20](#ic-20), [IC-18](#ic-18), [IC-19](#ic-19), [IC-21](#ic-21), [IC-22](#ic-22), [IC-23](#ic-23).

**Context:** A fixed package revision, which need not be fully validated.

**Minimum semantic content and multiplicity:** One resolved method/material/data/convention bundle, default eligibility and at least one explicit compatibility assessment, even if partly unassessed.

**Boundary:** No local stream state, live handle or implicit current-material context.

**Consumers retained from B5:** P04, P05, P07, P08, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A07. Region/location binding

**Semantic owner:** P07. **Primary concept:** [IC-49](#ic-49). **Decomposition:** [IC-49](#ic-49), [IC-48](#ic-48).

**Context:** An effective location assignment.

**Minimum semantic content and multiplicity:** Exactly one location, one package revision and a resolution trace; zero or more authorized local overlays and boundary prerequisites.

**Boundary:** Different locations can share a definition while retaining independent state and restrictions.

**Consumers retained from B5:** P04, P05, P08, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A08. Local material description

**Semantic owner:** P04. **Primary concept:** [IC-28](#ic-28). **Decomposition:** [IC-28](#ic-28), [IC-25](#ic-25), [IC-26](#ic-26), [IC-27](#ic-27), [IC-29](#ic-29), [IC-30](#ic-30), [IC-31](#ic-31), [IC-32](#ic-32), [IC-33](#ic-33), [IC-71](#ic-71).

**Context:** A local snapshot with known, absent and unresolved quantities explicitly distinguished.

**Minimum semantic content and multiplicity:** One material account; one or more composition descriptions; zero or more local domain entries and optional distribution/site/transfer descriptions.

**Boundary:** State, flow, inventory and reference amount are distinct; reporting views are non-additive.

**Consumers retained from B5:** P05, P07, P08, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A09. Calculation problem and authority contract

**Semantic owner:** P05. **Primary concept:** [IC-35](#ic-35). **Decomposition:** [IC-35](#ic-35), [IC-34](#ic-34), [IC-36](#ic-36), [IC-37](#ic-37), [IC-38](#ic-38), [IC-39](#ic-39), [IC-40](#ic-40), [IC-41](#ic-41), [IC-42](#ic-42).

**Context:** An original physical problem or intentionally partial mathematical demand.

**Minimum semantic content and multiplicity:** One operation contract, one captured package context, one constraint set and one authority allocation; one or more input descriptions and requested outputs/checks.

**Boundary:** Attempts, guesses and changed-physics alternatives never overwrite original request meaning.

**Consumers retained from B5:** P07, P08, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A10. Readiness assessment

**Semantic owner:** P03. **Primary concept:** [IC-24](#ic-24). **Decomposition:** [IC-24](#ic-24).

**Context:** A request-specific qualification at a captured context.

**Minimum semantic content and multiplicity:** One package/demand descriptor; a complete set of applicable prerequisite conclusions and supporting attestations; each unknown remains explicit.

**Boundary:** Ready to attempt is not converged or independently validated.

**Consumers retained from B5:** P07, P08, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A11. Provider realization/capability attestation

**Semantic owner:** P08. **Primary concept:** [IC-53](#ic-53). **Decomposition:** [IC-53](#ic-53), [IC-58](#ic-58).

**Context:** An exposed realization and its bounded capability facts.

**Minimum semantic content and multiplicity:** One exact engine/build/binding/data manifest; one or more realized operation mappings, native-coordinate and lifecycle restrictions where applicable.

**Boundary:** Upstream catalog or method signature does not establish adapter capability.

**Consumers retained from B5:** P03, P05, P07, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A12. Execution plan and numerical workspace

**Semantic owner:** P08. **Primary concept:** [IC-54](#ic-54). **Decomposition:** [IC-54](#ic-54), [IC-55](#ic-55), [IC-57](#ic-57), [IC-72](#ic-72).

**Context:** A planned or active numerical attempt.

**Minimum semantic content and multiplicity:** One physical problem, one realization and one numerical policy; zero or more initializer descriptions; active workspace/session association only when needed.

**Boundary:** Plan and disposable working memory differ; only the reproducible recipe/evidence belongs in the semantic archive.

**Consumers retained from B5:** P07. **Current-model commitment:** P10, without transferring semantic ownership.

### A13. Provider candidate and execution evidence

**Semantic owner:** P08. **Primary concept:** [IC-56](#ic-56). **Decomposition:** [IC-56](#ic-56).

**Context:** A terminated or intermediate candidate from an identified attempt.

**Minimum semantic content and multiplicity:** One producing attempt; zero or more normalized payloads; explicit per-output status, actual used assumptions, numerical evidence and actual exchanges.

**Boundary:** No automatic acceptance, complete-property guarantee or current-result authority.

**Consumers retained from B5:** P07, P09. **Current-model commitment:** P10, without transferring semantic ownership.

### A14. Qualification and acceptance evidence

**Semantic owner:** P09. **Primary concept:** [IC-60](#ic-60). **Decomposition:** [IC-60](#ic-60), [IC-59](#ic-59), [IC-62](#ic-62), [IC-63](#ic-63).

**Context:** An assessment against the original criteria and requested scope.

**Minimum semantic content and multiplicity:** One candidate/set and original problem/scope context; required checks each have pass/fail/unassessed/inapplicable evidence; lineage and precise outcome.

**Boundary:** Scientific/local acceptance does not establish currentness or parent-scope convergence.

**Consumers retained from B5:** P07, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A15. Current-result publication binding

**Semantic owner:** P10. **Primary concept:** [IC-67](#ic-67). **Decomposition:** [IC-67](#ic-67), [IC-66](#ic-66).

**Context:** A successful or rejected guarded publication and resulting current association.

**Minimum semantic content and multiplicity:** One proposal/decision with qualification, context and run guards; committed binding slots reference one coherent result set each; related slots update as a group.

**Boundary:** P10 owns current association, not the physical law or scientific judgment.

**Consumers retained from B5:** P04, P07, P09. **Current-model commitment:** P10, without transferring semantic ownership.

### A16. Change and reconstruction record

**Semantic owner:** P10. **Primary concept:** [IC-64](#ic-64). **Decomposition:** [IC-64](#ic-64), [IC-65](#ic-65), [IC-68](#ic-68), [IC-69](#ic-69), [IC-70](#ic-70).

**Context:** A revision/change/archive/reconstruction event or captured context.

**Minimum semantic content and multiplicity:** Explicit record variant: captured revision, proposed/committed change, archive, reconstruction/reproduction, or run authority. Required variant fields cannot be replaced by a universal property bag.

**Boundary:** A loaded document can be inspectable but not ready; runtime handles are not semantic content.

**Consumers retained from B5:** P01, P02, P03, P04, P06, P07, P08, P09. **Current-model commitment:** P10, without transferring semantic ownership.

### A17. Coverage and independent validation record

**Semantic owner:** P09. **Primary concept:** [IC-61](#ic-61). **Decomposition:** [IC-61](#ic-61).

**Context:** A bounded coverage claim and its evidence.

**Minimum semantic content and multiplicity:** One scenario/configuration/envelope claim; separate R/E/V/conformance/reproduction dimensions; actual witnesses only when they exist.

**Boundary:** Catalog completion cannot promote numerical execution, and unknown evidence cannot be a pass.

**Consumers retained from B5:** P03, P07, P08, P10. **Current-model commitment:** P10, without transferring semantic ownership.

### A18. Unit demands, coupling and routing description

**Semantic owner:** P07. **Primary concept:** [IC-50](#ic-50). **Decomposition:** [IC-50](#ic-50), [IC-51](#ic-51), [IC-52](#ic-52).

**Context:** The thermodynamic boundary of a chosen process formulation.

**Minimum semantic content and multiplicity:** One unit mode and participating location set; required property/equation demands; balance/coupling/routing rules and explicit completion hierarchy; optional translation case(s).

**Boundary:** Equipment geometry, global equation construction and process solve remain host responsibilities.

**Consumers retained from B5:** P05, P08, P09, P10. **Current-model commitment:** P10, without transferring semantic ownership.

## 4. Domain concept sheets

Common identity, quantity, completeness and reference rules in §1 apply to every sheet. Examples are authored semantic witnesses, not executed provider tests. Fields named as groups can later be stored by reference or computed; neither their order nor grouping prescribes a wire/storage layout.

### P01 — Material semantics and representations

<a id="ic-01"></a>
#### IC-01. Constituent identity

**Owner:** P01. **Kind:** definition. **Step-5 product:** A01.

What a represented constituent is, independently of the database name, numerical coordinate, local amount, or selected property method. A constituent may be a chemically identified species or an explicitly defined engineering surrogate.

**Identity:** Stable entity identity plus a revision of its defining description. A display-name change does not create another material; changing a surrogate's defining cut or blend can require a new definition revision or a distinct constituent, with an explicit equivalence decision.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Identity kind | 1 | Chemically identified, assay-defined cut, specified lump, empirical material, pseudo-pure surrogate, or another registered meaning. | Categorical meaning, not a required enumeration. |
| Defining description | 1 | Sufficient identity criteria for the kind; chemical structure/formula evidence where justified, or cut/assay/blend/empirical criteria otherwise. | Formula, composition basis, boiling-cut test basis, or other explicitly stated descriptors. |
| Assertions and aliases | 0..* | Supporting identities and names; ambiguities are retained until resolved. | Names are not conserved material quantities. |
| Conservation description | 0..1 | Justified mass, elemental, charge, site or other accounting meaning; absent chemical detail does not mean zero atoms. | Declared conserved basis; molecular weight only when justified. |
| Derivation origin | 0..* | Source or characterization proposal supporting this identity; a derived cut keeps its origin. | Provenance references. |

**Permitted variants:** A shared chemical identity with separate phase-state instances. An empirically defined mass constituent with no molecular description. A surrogate of a blend, explicitly not identical to the detailed blend representation.

**Invariants:** Do not synthesize molecular weight, formula, charge, or registry identifiers to satisfy a provider. A species identity does not require its own local amount to be positive. Solid form identity or a phase-specific species convention must remain distinguishable when the chosen model requires it.

**Lifecycle and mutability:** Draft assertions become an assessed identity proposal; P10 commits revisions. Existing results retain the identity revision they used.

**Related concepts:** [IC-02](#ic-02), [IC-05](#ic-05), [IC-10](#ic-10), [IC-17](#ic-17).

**Valid interpretation:** An assay-derived 180–220 degree boiling cut is identified by assay, test convention and cut definition, not given a fictional registry number.

**Invalid interpretation:** Two equal display names are merged despite different defining cut ranges.

**Requirement basis:** [FR-MAT-01](thermodynamics_functional_requirements_v0_1.md#fr-mat-01), [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05). **Global rules:** SI-01. **Worked examples:** SE-03, SE-07.

<a id="ic-02"></a>
#### IC-02. Identity assertion and alias resolution

**Owner:** P01. **Kind:** assertion. **Step-5 product:** A01.

A sourced statement connecting a name, external identifier, or identity claim to a constituent. The assertion may be unresolved or contested.

**Identity:** Assertion identity includes issuer/namespace, asserted value, source and scope; resolving the assertion creates evidence, not an unlogged overwrite of identity.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Namespace and asserted value | 1 | Provider-local name, registry identifier, local tag, formula claim or synonym and its namespace. | Text with explicit namespace; case/normalization rules declared. |
| Candidate constituents | 0..* | Possible interpretations. Exactly one target is required only when the assertion is declared resolved. | Identity references. |
| Evidence | 1..* | Source and context establishing or qualifying the association. | Provenance, not a numeric confidence inferred from a match. |
| Resolution status | 1 | Unresolved, unique, ambiguous, rejected or superseded association, including the reason. | Assessment vocabulary. |
| Applicability | 0..1 | Provider/version, project, dataset or time scope within which the alias is valid. | Explicit scope; no universal-name assumption. |

**Permitted variants:** A project-local alias may resolve only within one representation. An external registry assertion may be shared across configurations.

**Invariants:** A lookup with multiple candidates remains ambiguous. A provider name is never silently promoted to the canonical physical identity.

**Lifecycle and mutability:** Append resolution or supersession evidence; preserve the originally imported assertion.

**Related concepts:** [IC-01](#ic-01), [IC-10](#ic-10), [IC-15](#ic-15).

**Valid interpretation:** The local tag SOLVENT refers to a particular mixture definition in one project, not universally to a species.

**Invalid interpretation:** Choosing the first database hit and recording it as a verified identity.

**Requirement basis:** [FR-MAT-01](thermodynamics_functional_requirements_v0_1.md#fr-mat-01). **Global rules:** SI-01.

<a id="ic-03"></a>
#### IC-03. Material representation revision

**Owner:** P01. **Kind:** definition. **Step-5 product:** A01.

The constituent and coordinate vocabulary used to describe a material. It is not the physical inventory and is not the thermodynamic package.

**Identity:** Representation lineage plus exact revision. Constituent membership or interpretation changes produce a new revision; mere display ordering is not a physical change.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Constituent set | 1..* | Allowed constituent identities, including admissible reaction products with zero current amount. | Identity revisions, not amounts. |
| Coordinate descriptions | 1..* | Permitted amount/fraction and detailed/apparent/lumped coordinate descriptions. | Explicit bases and normalization domains. |
| Domain definitions | 1..* | Bulk, solid, nonbulk or empirical material-domain meanings usable by the representation. | Domain semantics; not a count of present phases. |
| Justified accounting bases | 1..* | At least one meaningful accounting basis; no universal requirement for chemical element data. | Mass, element amount, charge, sites or other justified conservation. |
| Maps to other representations | 0..* | Declared coordinate transformations, aggregations or chemical reconciliation links. | Map revisions. |
| Extension definitions | 0..* | Distribution and site/loading meanings; numerical support is separate. | Attribute coordinate, weighting and denominator semantics. |

**Permitted variants:** True-species composition. Apparent feed or reporting description. Petroleum compositional cuts, black-oil bulk description, mass-only empirical description.

**Invariants:** Two simultaneous descriptions of the same material must not create two additive inventories. A draft may have unresolved constituent identity, but cannot be declared fully resolved. Adding species changes coordinates and eligibility assessments; it is not merely resizing a numerical vector.

**Lifecycle and mutability:** P01 validates meaning; P10 adopts a representation revision and any coupled mapping/quantity changes.

**Related concepts:** [IC-01](#ic-01), [IC-04](#ic-04), [IC-05](#ic-05), [IC-06](#ic-06), [IC-07](#ic-07), [IC-08](#ic-08), [IC-09](#ic-09).

**Valid interpretation:** Apparent salt feed and true ionic representation are linked views of one material account.

**Invalid interpretation:** Summing apparent salt and its dissociated ions as independently present mass.

**Requirement basis:** [FR-MAT-02](thermodynamics_functional_requirements_v0_1.md#fr-mat-02), [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05), [FR-LIF-02](thermodynamics_functional_requirements_v0_1.md#fr-lif-02). **Global rules:** SI-01. **Worked examples:** SE-03, SE-07.

<a id="ic-04"></a>
#### IC-04. Composition coordinate system

**Owner:** P01. **Kind:** definition. **Step-5 product:** A01.

The meaning of a constituent vector or constrained coordinate chart, including which total and constituent set its numbers refer to.

**Identity:** Identity fixes representation revision, coordinate kind, ordering/chart and normalization domain. Equivalent permutations are recorded maps, not changed physical mixtures.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Representation | 1 | Constituent meanings to which the coordinates refer. | Exact representation revision. |
| Coordinates and order | 1..* | Ordered constituent references, independent variables or explicitly defined transformed coordinates. | Per-coordinate basis; no unlabeled vector. |
| Coordinate kind | 1 | Component amounts/rates, normalized fractions, ratios, or an independent composition chart. | Mole, mass, concentration, carrier ratio or declared basis. |
| Normalization and denominator | 1 | Whole mixture, named phase, solvent, dry carrier, or other stated domain; includes reference state if volume based. | A positive denominator is required for a numerical ratio, not for preserving a zero-flow feed composition specification. |
| Dependent-coordinate rule | 0..1 | For reduced charts, reconstruct dependent coordinates and state admissible domain; required whenever not all full coordinates are independent. | Example x_N=1−sum(x_1…x_(N−1)); logarithms need positive coordinates and a declared convention. |
| Conversion prerequisites | 0..* | Data and conventions needed for basis or chart conversion; unknown prerequisites block that conversion only. | Molecular weights, solvent mass, density model or chart Jacobian as applicable. |

**Permitted variants:** Full mass fraction vector with sum constraint. N−1 independent mole fractions. Component mass flows without a separately authoritative total.

**Invariants:** Fraction vectors are not independent coordinates unless a chart specifies their constraints. Provider permutations and scientific coordinate transformations are distinct. A concentration, molality and mole fraction are not interchangeable because all mention a species.

**Lifecycle and mutability:** Revise when coordinate meaning changes; preserve the old chart for derivative and result interpretation.

**Related concepts:** [IC-01](#ic-01), [IC-03](#ic-03), [IC-12](#ic-12), [IC-21](#ic-21).

**Valid interpretation:** A derivative chart fixes x_1 and x_2 as independent and defines x_3=1−x_1−x_2.

**Invalid interpretation:** A derivative with respect to x_1 silently holds every other mole fraction fixed while also requiring their sum to remain one.

**Requirement basis:** [FR-MAT-07](thermodynamics_functional_requirements_v0_1.md#fr-mat-07), [FR-STA-08](thermodynamics_functional_requirements_v0_1.md#fr-sta-08), [FR-PRP-05](thermodynamics_functional_requirements_v0_1.md#fr-prp-05). **Global rules:** SI-04, SI-05. **Worked examples:** SE-03, SE-06.

<a id="ic-05"></a>
#### IC-05. Conserved-quantity basis

**Owner:** P01. **Kind:** definition. **Step-5 product:** A01.

Which material quantities can be conserved and how they are calculated from a stated representation. Conservation is conditional on the physical operation and modeled exchanges.

**Identity:** Basis identity includes quantity kinds, representation, conversion rules and justified-data provenance; a label such as mass is not enough to interpret an arbitrary coordinate vector.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Representation and coordinates | 1 | Input coordinates on which accounting is defined. | Exact amount/rate basis. |
| Conserved quantities | 1..* | Named totals justified for this material; chemical elements/charge only when meaningful. | Mass, molar element amount, charge equivalents, sites or specified quantity dimensions. |
| Conversion rule | 1 | Linear matrix when justified, or explicitly declared nonlinear/empirical mapping with prerequisites. | For b=A n, each matrix entry carries target-units per input-amount-unit. |
| Evidence and completeness | 1 | Complete, partial or unassessed basis and the source of coefficients/conventions. | Missing rows or entries are not zeros. |
| Applicable transformations | 1..* | Operations under which a quantity is conserved and terms that can cross the boundary. | Reaction, phase transfer, mechanical split or open exchange distinctions. |

**Permitted variants:** Exact stoichiometric elemental accounting. Mass-only empirical accounting. Site inventory and surface species with explicit occupancy relations.

**Invariants:** A nonreactive phase split preserves constituent totals; chemistry need not preserve molecular mole count. Physical exchange changes account totals; conservation is checked on the system plus declared exchange. Missing elemental information cannot yield a passed elemental balance.

**Lifecycle and mutability:** P01 maintains meaning; assessment of an actual residual belongs to P09.

**Related concepts:** [IC-03](#ic-03), [IC-04](#ic-04), [IC-10](#ic-10), [IC-15](#ic-15).

**Valid interpretation:** A mass-only particulate model checks kg/s while declining elemental-balance claims.

**Invalid interpretation:** Treating unreported atoms of an empirical solid as zero and certifying an element balance.

**Requirement basis:** [FR-MAT-08](thermodynamics_functional_requirements_v0_1.md#fr-mat-08), [FR-CHM-06](thermodynamics_functional_requirements_v0_1.md#fr-chm-06), [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03). **Global rules:** SI-15. **Worked examples:** SE-03, SE-04.

<a id="ic-06"></a>
#### IC-06. Material-domain definition

**Owner:** P01. **Kind:** definition. **Step-5 product:** A01.

The kind of physical domain in which a material description is meaningful, without asserting a local phase exists or choosing an equilibrium model.

**Identity:** Domain-definition identity denotes semantics; local domain instances belong to P04 and process locations to P07.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Domain kind | 1 | Bulk fluid, identified solid or solid solution, empirical solid, surface/site inventory, membrane-side bulk, or explicit additional kind. | Qualitative physical meaning. |
| Admissible quantity bases | 1..* | Allowed stored amounts, flows, loadings or material attributes for that kind. | Extensive and intensive bases are stated separately. |
| Constituent interpretation | 1 | How constituents participate in the domain; distinction between physical species and reported surrogates. | Uses representation meanings. |
| Specialized attributes | 0..* | Distribution/site or other registered domain semantics required beyond ordinary amounts. | Defined support/denominator and transfer meaning. |

**Permitted variants:** One domain definition can describe many liquid instances. A membrane interface may relate two domains without being an independent stored material amount.

**Invariants:** A domain category is not a runtime phase instance. Bulk phase, surface, interface and equipment compartment are not automatically synonymous. Do not infer transfer, kinetics or equilibrium from the domain label.

**Lifecycle and mutability:** Definitions are reusable; local creation/deletion does not mutate their meaning.

**Related concepts:** [IC-03](#ic-03), [IC-04](#ic-04), [IC-08](#ic-08), [IC-09](#ic-09).

**Valid interpretation:** Two distinct liquids share liquid-domain semantics but have separate compositions and result-local identities.

**Invalid interpretation:** Creating a fake liquid to store an adsorbed surface amount with no defined liquid volume.

**Requirement basis:** [FR-MAT-02](thermodynamics_functional_requirements_v0_1.md#fr-mat-02), [FR-CFG-07](thermodynamics_functional_requirements_v0_1.md#fr-cfg-07), [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01). **Global rules:** SI-09, SI-27.

<a id="ic-07"></a>
#### IC-07. Representation map and reduction

**Owner:** P01. **Kind:** definition. **Step-5 product:** A02.

A declared relation between two descriptions of material: permutation, basis conversion, aggregation, constrained reconstruction, or a map requiring a separate chemical solution.

**Identity:** Map revision identifies source/target representations, direction, prerequisites, rule and information-loss statement. A reverse relation is not implied.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Source representation | 1 | Domain and coordinates of the original description. | Representation revision and input basis. |
| Target representation | 1 | Domain and coordinates of the resulting description. | Representation revision and output basis. |
| Mapping rule | 1 | Exact linear rule, conditional nonlinear conversion, aggregation or constraints requiring a solve. | Units per coefficient or complete functional convention. |
| Conservation claims | 1..* | Which justified totals are preserved, conditionally preserved or unassessed. | Input and output accounting bases. |
| Information loss and inverse | 1 | Retained/lost distinctions; inverse available, conditional, many-to-one, or unavailable. | Additional reconstruction data or prior-policy assumptions explicitly named. |
| Applicability and dependencies | 0..* | States, data, convention or chemistry prerequisites of the map. | No fixed map asserted for state-dependent speciation. |

**Permitted variants:** Invertible permutation or units/basis conversion with data. Many-to-one lumping or moment reduction. Constraint relation delegated to an explicit speciation operation.

**Invariants:** For a linear amount map n_t=M n_s, claimed exact conservation requires A_t M=A_s on the stated domain. If chemical reactions make reverse reporting nonunique, report the selected attribution convention rather than inventing unique molecules. A provider permutation preserves meaning; a lumping map can destroy information.

**Lifecycle and mutability:** Revisioned definition; each application keeps its own source snapshot, assumptions and result.

**Related concepts:** [IC-03](#ic-03), [IC-04](#ic-04), [IC-05](#ic-05), [IC-15](#ic-15), [IC-21](#ic-21), [IC-47](#ic-47).

**Valid interpretation:** Combining masses of A and B into one lump preserves total mass but records the lost A:B split.

**Invalid interpretation:** Recovering a unique A:B composition from lump mass alone.

**Requirement basis:** [FR-MAT-07](thermodynamics_functional_requirements_v0_1.md#fr-mat-07), [FR-MAT-08](thermodynamics_functional_requirements_v0_1.md#fr-mat-08), [FR-CHM-06](thermodynamics_functional_requirements_v0_1.md#fr-chm-06), [FR-FLW-06](thermodynamics_functional_requirements_v0_1.md#fr-flw-06), [FR-EXT-02](thermodynamics_functional_requirements_v0_1.md#fr-ext-02). **Global rules:** SI-05, SI-14. **Worked examples:** SE-03, SE-04, SE-08.

<a id="ic-08"></a>
#### IC-08. Distributed attribute definition

**Owner:** P01. **Kind:** definition. **Step-5 product:** A01.

The meaning of a distribution or moments attached to material, such as a chain-length or particle-size description. It is not simply an arbitrary list of numbers.

**Identity:** Definition fixes support coordinate, weighting measure, normalization, attribute scope and permitted reductions.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Support coordinates | 1..* | Chain length, size, mass or other defined coordinate and its domain/bins. | Coordinate units; discrete masses versus probability densities distinguished. |
| Weighting and normalization | 1 | Number-, mole-, mass- or another justified measure; total measure or normalized distribution. | Integral/sum convention and denominator. |
| Material association | 1 | Which constituent/domain or representation the attribute qualifies. | Definition references, not local quantity. |
| Reduction rules | 0..* | Moments/statistics retained and information lost; normalization and weighting are part of each moment. | Moment dimensions depend on support and weighting. |
| Transformation assumptions | 1..* | Permitted mixing, transfer and moment conversions and required conversion data. | No automatic conservation of every moment under reactions or size change. |

**Permitted variants:** Discrete binned distribution. Continuous density with support measure. Finite moments with explicit lost information.

**Invariants:** A number-weighted mean and mass-weighted mean remain different observables. A probability density has coordinate-dependent units; bin weights are not densities. A reduced moment set does not specify a unique full distribution without additional assumptions.

**Lifecycle and mutability:** Reusable meaning; local attribute values appear in IC-32 and inherit their own result/input roles.

**Related concepts:** [IC-01](#ic-01), [IC-03](#ic-03), [IC-06](#ic-06), [IC-07](#ic-07).

**Valid interpretation:** A mass-weighted particle-size histogram is mixed using material mass weights over reconciled bins.

**Invalid interpretation:** Averaging two normalized histograms equally despite unequal associated masses.

**Requirement basis:** [FR-EXT-02](thermodynamics_functional_requirements_v0_1.md#fr-ext-02). **Global rules:** SI-27. **Worked examples:** SE-08.

<a id="ic-09"></a>
#### IC-09. Surface/site and loading basis

**Owner:** P01. **Kind:** definition. **Step-5 product:** A01.

The denominator, capacity, species occupancy and conserved-material interpretation of an adsorbed or other nonbulk quantity.

**Identity:** Identity fixes loading convention, domain and occupancy relationships. Geometry/amount values belong to process context or local state, not this reusable meaning.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Domain and constituent meaning | 1 | Surface/site domain and relevant constituent definitions. | Domain and representation revision. |
| Loading denominator | 1 | Surface area, dry sorbent mass, site amount or other explicitly defined denominator. | For example mol/kg dry sorbent or mol/m²; wet and dry masses are not interchangeable. |
| Occupancy and capacity relation | 0..1 | Required when capacity/site constraints are claimed; relation of occupied/vacant sites and species amounts. | Stoichiometric site occupancy and capacity units. |
| Transfer accounting rule | 1 | How loading converts to actual stored material and changes on exchange. | Amount=loading×actual denominator only under the declared basis relation. |
| Prerequisite context | 1..* | Denominator values and provider/model inputs required at a local location. | No assumed unit area or unit sorbent mass as physical fact. |

**Permitted variants:** Mass-normalized adsorption loading. Area-normalized excess quantity. Site occupancy with explicit multidentate stoichiometry.

**Invariants:** Loading without its denominator cannot establish total stored amount. Vacant-site bookkeeping is not additional bulk chemical mass unless the model explicitly defines it. Surface capacity is not automatically a thermodynamic phase fraction.

**Lifecycle and mutability:** P3 semantics defined; actual adsorption/membrane numerical execution remains a separate capability.

**Related concepts:** [IC-03](#ic-03), [IC-05](#ic-05), [IC-06](#ic-06), [IC-07](#ic-07).

**Valid interpretation:** 0.2 mol/kg dry sorbent at 5 kg dry sorbent corresponds to 1 mol stored under the defined total-loading convention.

**Invalid interpretation:** Reading 0.2 mol/kg as 0.2 mol without sorbent quantity.

**Requirement basis:** [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01). **Global rules:** SI-27. **Worked examples:** SE-09.

### P02 — Property evidence, parameters and characterization

<a id="ic-10"></a>
#### IC-10. Evidence-source record

**Owner:** P02. **Kind:** evidence. **Step-5 product:** A03.

An attributable source for identity, property, parameter, model or validation statements. Presence of a citation is not a quality endorsement.

**Identity:** Source identity fixes edition/artifact/locator and provenance. A moving URL alone cannot identify the exact data used.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Source identity and locator | 1 | Document/database/artifact identifier, version or snapshot, and relevant location. | Attribution metadata. |
| Recorded artifact evidence | 0..1 | Checksum or archived snapshot where accessible; otherwise explicit reproduction limitation. | Artifact identity, not scientific truth. |
| Original conventions | 0..* | Units, definitions, conditions and reported interpretation retained from the source. | Do not rewrite historical source conventions to current defaults. |
| Quality and uncertainty statements | 0..* | What the source actually establishes, uncertainty, method and evaluation status. | Unknown uncertainty is not zero. |
| Access/dependency conditions | 0..* | Known data availability/redistribution constraints and missing-resource facts. | No independent legal certification implied. |

**Permitted variants:** Measured-data publication. Pinned provider-data bundle with partially opaque internals. User assertion with limited supporting evidence.

**Invariants:** Do not label library data critically evaluated without evidence. A source artifact, a derived value and an acceptance test are different records.

**Lifecycle and mutability:** Preserve prior snapshots; later source revisions add new evidence and do not rewrite historical provenance.

**Related concepts:** .

**Valid interpretation:** A fitted coefficient links to an exact input dataset and method record.

**Invalid interpretation:** A bare current database name replaces the original source of a historical result.

**Requirement basis:** [FR-DAT-01](thermodynamics_functional_requirements_v0_1.md#fr-dat-01), [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07).

<a id="ic-11"></a>
#### IC-11. Property datum and observation

**Owner:** P02. **Kind:** evidence. **Step-5 product:** A03.

A measured, reported or externally supplied property datum before selecting it for a model. It retains conditions and uncertainty rather than being mistaken for a universal constant.

**Identity:** Observation identity belongs to its source/sample/measurement or imported record. Replicates are separate observations unless explicitly aggregated.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Observed quantity meaning | 1 | Property, material/domain, basis, original units and relevant convention declared as source descriptors. | Scalar, vector or tensor axes; no dependency on a live evaluation request. |
| Material and experimental context | 1 | Constituents/composition and known phase/experimental conditions; unresolved context labeled. | Stated T,P,composition,basis and measurement conditions. |
| Value or interval | 1 | Reported value, bounds or censored observation, not a missing value filled by zero. | Original quantity units and any explicit conversion record. |
| Uncertainty and correlation | 0..* | Reported standard/expanded/interval/unknown uncertainty; coverage and correlation-group information where known. | Uncertainty dimensions/covariance units and coverage factors as stated. |
| Source and validity | 1..* | Attribution and documented conditions under which the datum is applicable. | Source bounds distinct from model fit or solver search bounds. |

**Permitted variants:** Single point with uncertainty. Interval/censored value. Correlated experimental series with shared covariance information.

**Invariants:** Unavailable uncertainty must not imply independent exact observations. A temperature-dependent datum is not a constant merely because it is stored as one number.

**Lifecycle and mutability:** Immutable source observation; normalization or regression produces separately attributable derivatives/proposals.

**Related concepts:** [IC-01](#ic-01), [IC-03](#ic-03), [IC-10](#ic-10), [IC-15](#ic-15).

**Valid interpretation:** A binary VLE observation retains both phase compositions and reported temperature/pressure.

**Invalid interpretation:** Dropping the measured phase designation and using the value for any phase.

**Requirement basis:** [FR-DAT-01](thermodynamics_functional_requirements_v0_1.md#fr-dat-01), [FR-DAT-07](thermodynamics_functional_requirements_v0_1.md#fr-dat-07).

<a id="ic-12"></a>
#### IC-12. Parameter interpretation descriptor

**Owner:** P02. **Kind:** definition. **Step-5 product:** A03.

The meaning of a parameter or parameter group required by a method, including its equation convention and how absence is interpreted. P03 supplies method needs in this vocabulary.

**Identity:** Identity fixes the method-specific formula/convention, roles and coordinate orientation, not just a coefficient name.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Method and formula descriptor | 1 | Explicit method identity/convention or attributable inseparable-bundle descriptor; includes temperature functional form. | Descriptor values, not a runtime package reference. |
| Subject roles | 1..* | Pure constituent, ordered pair, symmetric pair, group pair, site association or other roles. | Ordered roles remain ordered unless symmetry is declared. |
| Parameter components | 1..* | Coefficient names and exact formula roles; parameter groups can be inseparable. | Units for each coefficient and independent-variable units. |
| Missing-value policy | 1 | Required, rule-derived, omitted interaction, documented default, or unsupported; no generic zero rule. | Policy identity plus model meaning. |
| Interpretation prerequisites | 0..* | Reference conventions, support domain, group definitions or required context. | Functional arguments and their allowed coordinates. |

**Permitted variants:** Temperature-independent symmetric cubic pair coefficient. Directed activity-model coefficient set. Provider-bundled parameter interpretation with stated opacity.

**Invariants:** Same coefficient labels in different formulas are not interchangeable. Directed interactions cannot be silently symmetrized. A user-specified zero is not missing data; an omitted interaction is a physical meaning.

**Lifecycle and mutability:** Revision when formula, units, roles or missing-value meaning changes.

**Related concepts:** [IC-01](#ic-01), [IC-15](#ic-15).

**Valid interpretation:** An A+B/T expression records that A is dimensionless and B has temperature units under its stated formula.

**Invalid interpretation:** Copying A12 from a different parameter convention solely because its column has the same name.

**Requirement basis:** [FR-DAT-02](thermodynamics_functional_requirements_v0_1.md#fr-dat-02). **Global rules:** SI-11. **Worked examples:** SE-11.

<a id="ic-13"></a>
#### IC-13. Parameter value assertion

**Owner:** P02. **Kind:** assertion. **Step-5 product:** A03.

One candidate value or value group under a precise parameter interpretation, with source, validity, derivation and approval status kept separate.

**Identity:** Assertion identity fixes interpretation, subjects, source/derivation and value revision. Several competing assertions can coexist.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Interpretation and subjects | 1 | Parameter descriptor and actual constituent/group/site role bindings. | Role-specific exact identity revisions. |
| Value state and payload | 1 | Explicit known-zero, known-nonzero, missing, rule-derived, omitted-interaction or substituted value state; payload only where meaningful. | Descriptor units and function convention. |
| Evidence and derivation | 1..* | Source, fitting trial, estimation or explicit user choice. | Measured, fitted, estimated and substituted origins distinguished. |
| Validity and uncertainty | 0..* | Applicability, uncertainty and correlation with other parameters where established. | Missing uncertainty remains unknown. |
| Selection/approval role | 1 | Candidate, trial, selected, approved or superseded role and authorizing record where applicable. | Approval does not establish physical validation. |

**Permitted variants:** An exact explicitly chosen zero. Rule-derived pair value with recorded combining rule. User-authorized substitute with a bounded applicability claim.

**Invariants:** A failed estimate does not silently install near-ideal coefficients in an existing approved set. Correlated fit coefficients retain joint uncertainty evidence when available.

**Lifecycle and mutability:** New assertion for new values; selection into snapshots is explicit and trial state does not alter approved data.

**Related concepts:** [IC-01](#ic-01), [IC-10](#ic-10), [IC-12](#ic-12), [IC-15](#ic-15), [IC-17](#ic-17), [IC-65](#ic-65).

**Valid interpretation:** A missing directed interaction remains unresolved while a deliberate zero interaction is evaluable under its descriptor.

**Invalid interpretation:** Storing both missing and specified-zero as 0 and later claiming identical provenance.

**Requirement basis:** [FR-DAT-01](thermodynamics_functional_requirements_v0_1.md#fr-dat-01), [FR-DAT-02](thermodynamics_functional_requirements_v0_1.md#fr-dat-02), [FR-DAT-03](thermodynamics_functional_requirements_v0_1.md#fr-dat-03), [FR-DAT-07](thermodynamics_functional_requirements_v0_1.md#fr-dat-07). **Global rules:** SI-11. **Worked examples:** SE-11.

<a id="ic-14"></a>
#### IC-14. Resolved parameter snapshot

**Owner:** P02. **Kind:** definition. **Step-5 product:** A03.

The fixed selected data supplied to a package or explicit trial, including the selection/override precedence and known unresolved needs.

**Identity:** Exact snapshot revision plus referenced interpretation/value/source revisions. A separate content fingerprint may support comparison but is not itself semantic equivalence.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Selected assertions or bundles | 0..* | Actual assertions or attributable provider-data bundles; zero only when the chosen method explicitly requires no external parameter data. Required-but-missing data are unresolved needs, not a parameter-free declaration. | Resolved units/conventions; opaque content allowed only with bounded claims. |
| Selection and override precedence | 1 | Deterministic rule selecting alternatives, including user overrides and authorized substitution. | Decision provenance, not numerical call order. |
| Material applicability | 1..* | Representation or constituent scope for which selections are made. | Identity/coordinate revisions. |
| Unresolved needs | 0..* | Explicit missing prerequisites for affected operations; resolved here means fixed choices, not universal numerical completeness. | Request-specific missing-data descriptors. |
| Approval/trial role | 1 | Approved use, provisional trial or fixed but unqualified selection. | Revision linkage and intended use. |

**Permitted variants:** Fully visible selected coefficients. Pinned inseparable provider data. Limited-property snapshot with explicit unresolved needs.

**Invariants:** Evaluation cannot re-resolve a moving database entry without a new snapshot. One incomplete property family need not invalidate independent supported quantities. A trial snapshot may be fixed and evaluable without being approved production data.

**Lifecycle and mutability:** Freeze for calculation; new selections create a new revision. P10 commits approved references, not provider caches.

**Related concepts:** [IC-03](#ic-03), [IC-10](#ic-10), [IC-12](#ic-12), [IC-13](#ic-13), [IC-17](#ic-17), [IC-65](#ic-65).

**Valid interpretation:** The exact caloric correlations selected for a heater remain available after a source database update.

**Invalid interpretation:** Reloading a saved case selects today's default coefficients while labeling the result a reproduction.

**Requirement basis:** [FR-DAT-01](thermodynamics_functional_requirements_v0_1.md#fr-dat-01), [FR-DAT-04](thermodynamics_functional_requirements_v0_1.md#fr-dat-04), [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05), [FR-DAT-06](thermodynamics_functional_requirements_v0_1.md#fr-dat-06), [FR-CFG-04](thermodynamics_functional_requirements_v0_1.md#fr-cfg-04), [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07). **Global rules:** SI-11, SI-12. **Worked examples:** SE-07, SE-11.

<a id="ic-15"></a>
#### IC-15. Validity and applicability description

**Owner:** P02. **Kind:** definition. **Step-5 product:** A03.

A qualified statement about the domain of source data, a fitted relationship or a model assumption. It is distinct from numerical solver bounds.

**Identity:** Identity fixes the subject being qualified, coordinates, domain rule, evidence and status. Unknown coverage is an explicit alternative.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Qualified subject | 1 | Datum, parameter group, material characterization or supplied model-applicability statement. | Exact revision reference or exported subject descriptor. |
| Coordinates and restrictions | 1..* | T,P,composition, phase, concentration, species set or other relevant axes and admissible region. | Units, closed/open endpoints and chart/basis; domain need not be rectangular. |
| Evidence and confidence of domain | 1 | Measured range, validated envelope, recommended applicability, inferred restriction or unknown coverage. | No fabricated uncertainty or global validity. |
| Extrapolation treatment | 1 | Allowed-with-label, forbidden or not assessed for this use; process authorization kept separately. | Policy and source of decision. |

**Permitted variants:** Explicit multidimensional domain. A conservative applicability rule. Unknown with required later assessment.

**Invariants:** A Newton bracket or temperature search limit is not a scientific validity range. Source measurement range and model validation range may differ and remain separately labeled.

**Lifecycle and mutability:** Add new evidence as a new assessment revision; historical results retain their original qualification.

**Related concepts:** [IC-04](#ic-04), [IC-10](#ic-10).

**Valid interpretation:** Coefficients fitted between 300 and 350 K retain that data range even if the numerical routine searches 200–600 K.

**Invalid interpretation:** Calling a successful 500 K solve validated because it lies within the solver bounds.

**Requirement basis:** [FR-DAT-07](thermodynamics_functional_requirements_v0_1.md#fr-dat-07). **Global rules:** SI-13.

<a id="ic-16"></a>
#### IC-16. Characterization and fitting recipe

**Owner:** P02. **Kind:** definition. **Step-5 product:** A04.

The reproducible evidence-to-definition task that generates surrogate constituents or parameter values; it is not a new live material configuration until adopted.

**Identity:** Recipe revision identifies original evidence, method, discretization/cuts/objective, conventions and requested outputs.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Input evidence | 1..* | Original assays, curves, light-end information or observations used for the task. | Original test/basis conventions preserved. |
| Method and target description | 1 | Characterization/estimation/fitting method, constraints, cut boundaries or objective/residual definitions. | All cut axes, objective weights and normalizations declared. |
| Target meaning and parameter needs | 1..* | Intended constituent representations and parameter interpretation contracts. | No generated numerical fields without semantic meaning. |
| Thermodynamic evaluation requirements | 0..* | Exported evaluation descriptors needed by the recipe; execution supplied by coordinator, not a live recursive data lookup. | Fixed trial data and conditions required for each evaluation. |
| Output and loss declaration | 1 | Expected proposed constituents/parameters and evidence; retained/lost characterization detail. | Assay reconciliation and approximation assumptions. |

**Permitted variants:** Assay cut generation. Pure-property or binary-parameter regression. Fixed empirical correlation selection without a numerical fit.

**Invariants:** A recipe without its raw inputs cannot reconstruct the characterization. Trial evaluation cannot silently estimate the parameters currently being fitted.

**Lifecycle and mutability:** Recipe remains fixed within a campaign; changing assumptions starts a distinguishable task revision.

**Related concepts:** [IC-03](#ic-03), [IC-10](#ic-10), [IC-11](#ic-11), [IC-12](#ic-12).

**Valid interpretation:** An assay recipe specifies test basis, cut grid and estimation methods and keeps original curve points.

**Invalid interpretation:** Saving only generated cut names while claiming the original assay can be reconstructed exactly.

**Requirement basis:** [FR-DAT-04](thermodynamics_functional_requirements_v0_1.md#fr-dat-04), [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05), [FR-DAT-06](thermodynamics_functional_requirements_v0_1.md#fr-dat-06), [FR-LIF-04](thermodynamics_functional_requirements_v0_1.md#fr-lif-04). **Global rules:** SI-12. **Worked examples:** SE-11.

<a id="ic-17"></a>
#### IC-17. Characterization or fit proposal and trials

**Owner:** P02. **Kind:** proposal. **Step-5 product:** A04.

The candidate outputs and assessment from a preparation campaign, including unsuccessful trials. It packages a coordinated proposal, not permission to mutate approved definitions.

**Identity:** Campaign/proposal identity plus trial identities and captured recipe/input revisions.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Recipe and input snapshot | 1 | Exact task, source evidence and assumptions used. | Recipe/data identity. |
| Trials and proposed values | 0..* | Candidate parameter/constituent descriptors and evaluation outcomes; zero proposals is valid for a failed campaign. | Declared units/bases; no accepted-state implication. |
| Fit/characterization assessment | 1 | Residual/objective values, feasibility, reconciliation, uncertainty if assessed, and unresolved needs. | Metric definition, scale and evidence source. |
| Selected proposal | 0..1 | Explicit proposed coordinated changes and why selected; absent on failure. | References to future material/data changes, not a current binding. |
| Evidence lineage | 1..* | Sources and execution-evidence references for trial evaluations. | Attribution not independent validation. |

**Permitted variants:** Successful bounded fit. Unsuccessful fit with useful residual diagnostics. Manual characterized-material proposal with explicit assumptions.

**Invariants:** Failed trials cannot corrupt selected production data. Parameter confidence estimates and empirical validation are separate evidence. A cut identity and its data are proposed together; partial adoption cannot silently leave incompatible references.

**Lifecycle and mutability:** P02 interprets proposals; P03 qualifies method completeness; P10 commits only domain-approved coordinated revisions.

**Related concepts:** [IC-10](#ic-10), [IC-13](#ic-13), [IC-14](#ic-14), [IC-16](#ic-16), [IC-65](#ic-65).

**Valid interpretation:** A rejected fit remains in campaign history while the old approved interaction set stays current.

**Invalid interpretation:** A trial optimizer writes directly into every stream's shared interaction table.

**Requirement basis:** [FR-DAT-03](thermodynamics_functional_requirements_v0_1.md#fr-dat-03), [FR-DAT-04](thermodynamics_functional_requirements_v0_1.md#fr-dat-04), [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05), [FR-DAT-06](thermodynamics_functional_requirements_v0_1.md#fr-dat-06). **Global rules:** SI-12. **Worked examples:** SE-11.

### P03 — Thermodynamic methods and configured packages

<a id="ic-18"></a>
#### IC-18. Thermodynamic method definition

**Owner:** P03. **Kind:** definition. **Step-5 product:** A06.

The physical and mathematical meaning of a property, phase, caloric, transport or effective-property method independently of local state and numerical implementation.

**Identity:** Method identity plus formulation/convention revision. A method family name alone does not establish a complete formula or compatible parameters.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Method role and formulation | 1 | EOS, activity, caloric, standard-state, transport, interface, effective closure or an inseparable provider method bundle. | Formula or attributable formulation descriptor; no requirement to expose symbolic equations. |
| Input and output meanings | 1..* | Required state coordinates, material domains, physical quantities and promised observables. | Units/bases and relevant axes; exported semantic descriptions. |
| Parameter obligations | 0..* | Required interpretation descriptors; none required for parameter-free relations. | Exact formula and coefficient conventions. |
| Convention and compatibility needs | 1..* | Caloric/chemical/reference and phase-method requirements; dependencies on other methods declared. | Convention references, not equal-unit assumptions. |
| Applicability evidence | 0..* | Applicable material/domain/conditions and known limits. | Scientific validity separate from solver bounds. |

**Permitted variants:** Visible constitutive formula. Composite-method role. Pinned whole-provider model with limited internal visibility.

**Invariants:** NRTL-like activity behavior and a cubic EOS are different method roles, not interchangeable package labels. A correlation-based material is not required to expose a thermodynamic potential. An effective property identifies its closure instead of masquerading as a unique phase property.

**Lifecycle and mutability:** Revise on physical formulation or convention change; selecting another equivalent implementation is separately recorded and qualified.

**Related concepts:** [IC-12](#ic-12), [IC-15](#ic-15), [IC-21](#ic-21).

**Valid interpretation:** A vapor fugacity method and liquid activity method state the standard-state relationship needed to use them together.

**Invalid interpretation:** Connecting matching function signatures and declaring thermodynamic consistency without convention checks.

**Requirement basis:** [FR-GOV-04](thermodynamics_functional_requirements_v0_1.md#fr-gov-04), [FR-CFG-01](thermodynamics_functional_requirements_v0_1.md#fr-cfg-01), [FR-CFG-02](thermodynamics_functional_requirements_v0_1.md#fr-cfg-02), [FR-PRP-04](thermodynamics_functional_requirements_v0_1.md#fr-prp-04), [FR-PRP-07](thermodynamics_functional_requirements_v0_1.md#fr-prp-07). **Global rules:** SI-19.

<a id="ic-19"></a>
#### IC-19. Reusable package recipe

**Owner:** P03. **Kind:** definition. **Step-5 product:** A06.

An engineer-facing selection of thermodynamic methods and preparation rules that may still have unresolved data or defaults.

**Identity:** Recipe lineage/revision is distinct from each resolved package revision derived from it.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Material applicability | 1 | Intended representations or material-selection constraints. | Representation identity or explicit admissibility rule. |
| Method selections | 1..* | Chosen roles, selectable alternatives or whole-provider bundle choice. | Method definitions, not live sessions. |
| Data selection rules | 1..* | Sources, overrides, estimation authorization and selection requirements. | Rules are not a resolved numerical snapshot. |
| Conventions and physical defaults | 1 | Reference/caloric choices and allowed local specializations. | Explicit defaults remain draft until resolved/accepted. |
| Unresolved choices | 0..* | Missing selections, conflicts or data needs that prevent specific operations. | Resolution status per choice. |

**Permitted variants:** A parameterized template for several slates. A whole-engine package choice with pinned bundle requirements.

**Invariants:** An unresolved recipe cannot be used as if it were an exact reproducible numerical configuration. A named default becomes a consequential physical choice only when explicitly resolved.

**Lifecycle and mutability:** Author/edit recipe; resolve into IC-20 with fixed selections. Past IC-20 records are not mutated by later recipe edits.

**Related concepts:** [IC-03](#ic-03), [IC-12](#ic-12), [IC-14](#ic-14), [IC-18](#ic-18), [IC-21](#ic-21), [IC-22](#ic-22).

**Valid interpretation:** A conventional gamma–phi recipe leaves binary data selection open until the material slate is known.

**Invalid interpretation:** Using the same recipe name as proof that two differently resolved parameter sets are the same package.

**Requirement basis:** [FR-CFG-01](thermodynamics_functional_requirements_v0_1.md#fr-cfg-01).

<a id="ic-20"></a>
#### IC-20. Resolved thermodynamic package revision

**Owner:** P03. **Kind:** definition. **Step-5 product:** A06.

The fixed coherent physical-method/data/reference choices available for use at one or many locations, independent of runtime session state.

**Identity:** Lineage plus exact revision referencing immutable dependencies. Semantic equivalence across revisions or providers requires an explicit assessment; a hash or shared label is insufficient.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Material representation | 1..* | Precisely admitted material representation revisions and any permitted mappings. | Identity, coordinates and justified bases. |
| Resolved methods | 1..* | Actual chosen phase, caloric, transport/effective or whole-bundle methods. | No unresolved alternative used implicitly. |
| Resolved data | 1..* | Fixed visible parameter snapshot or attributable inseparable bundle. | Data revisions and opacity limits. |
| Conventions and eligibility | 1 | Selected energy/standard states and default permitted physical domains/transformations. | Definitions, not numerical search settings. |
| Optional chemistry | 0..* | Coherent chemical-system definitions where needed. | Species and thermochemical compatibility. |
| Compatibility assessments | 1..* | Decisions supporting the chosen combinations and explicit unassessed restrictions. | Evidence/assessment; resolved does not mean independently validated. |
| Recipe origin | 0..1 | Source recipe where one was used; direct explicit construction also permitted. | Lineage. |

**Permitted variants:** Fully visible assembled package. Whole-provider bundle with bounded disclosed semantics. Explicit limited-property empirical package.

**Invariants:** Changing data, material meaning, caloric convention or physical method changes the effective definition. No local temperature, flow, trial phase amount or mutable native handle is a package definition. Transport completeness is demand-specific; an unresolved diffusivity does not erase supported enthalpy.

**Lifecycle and mutability:** Resolve/freeze and qualify; P10 commits approved references. Local overlays specialize IC-49/IC-35 without silently changing the shared base.

**Related concepts:** [IC-03](#ic-03), [IC-07](#ic-07), [IC-14](#ic-14), [IC-18](#ic-18), [IC-19](#ic-19), [IC-21](#ic-21), [IC-22](#ic-22), [IC-23](#ic-23), [IC-45](#ic-45).

**Valid interpretation:** Ten column stages share the same resolved package but retain independent state descriptions.

**Invalid interpretation:** Reassigning a native current-stream pointer changes the material that a supposedly immutable package means.

**Requirement basis:** [FR-GOV-04](thermodynamics_functional_requirements_v0_1.md#fr-gov-04), [FR-CFG-01](thermodynamics_functional_requirements_v0_1.md#fr-cfg-01), [FR-CFG-04](thermodynamics_functional_requirements_v0_1.md#fr-cfg-04), [FR-LIF-04](thermodynamics_functional_requirements_v0_1.md#fr-lif-04). **Global rules:** SI-02, SI-12, SI-28. **Worked examples:** SE-01, SE-07, SE-11, SE-15.

<a id="ic-21"></a>
#### IC-21. Energy, standard-state and reference convention set

**Owner:** P03. **Kind:** definition. **Step-5 product:** A06.

The conventions required to interpret and combine caloric, activity, fugacity and chemical-potential quantities. Caloric zero and chemical standard state are separate entries, not one universal reference flag.

**Identity:** Convention-set revision identifies material/phase scope, temperature/pressure conditions, amount basis, included contributions and transformation evidence.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Caloric reference specification | 1 | Enthalpy, entropy and internal-energy references where supported; contributions included or omitted and phase/constituent dependence. | Explicit amount basis, reference conditions and reference functions; unsupported members labeled. |
| Chemical standard-state specification | 0..* | Required for activity/fugacity/chemical-potential use: standard species/phase/solvent/dilution/pressure convention and definitions of dimensionless ratios. | Activities dimensionless under their stated standard; fugacity magnitude and standard pressure distinguished. |
| Formation and reaction accounting | 1 | What formation contribution is already in material energy and what correction the selected balance formulation needs. | Per-species/extent conventions; no blanket add-heat-of-reaction rule. |
| Permitted reference transformations | 0..* | Evidence-backed transform functions and applicable material/phase range, including composition-dependent corrections. | Example H_b=H_a+sum(n_i c_i) for declared molar offsets; no universal scalar assumed. |
| Compatibility evidence | 0..* | Why conventions can be combined or transformed; unknown relation remains unknown. | Evidence references. |

**Permitted variants:** One coherent potential-based convention. Correlation-based caloric methods with explicitly reconciled phase references. Limited empirical sensible-enthalpy convention with unsupported entropy/chemical potential.

**Invariants:** Identical units do not imply identical energy or chemical standard states. A reference correction is not a model-error correction or an undeclared heat exchange. Species-dependent energy shifts that do not cancel through reaction require consistent thermochemical reconciliation. A mere scalar Cp mixture rule is not necessarily the derivative of an equilibrated material enthalpy.

**Lifecycle and mutability:** Revise when selected conventions change; dependent sessions and results require reassessment. Historical values retain their original convention.

**Related concepts:** [IC-07](#ic-07), [IC-10](#ic-10), [IC-15](#ic-15), [IC-43](#ic-43), [IC-45](#ic-45).

**Valid interpretation:** Two material models with a known component-wise reference difference report that transformation separately from remaining enthalpy disagreement.

**Invalid interpretation:** Changing ionic standard states while leaving equilibrium constants and chemical potentials silently unadjusted.

**Requirement basis:** [FR-CFG-01](thermodynamics_functional_requirements_v0_1.md#fr-cfg-01), [FR-CFG-02](thermodynamics_functional_requirements_v0_1.md#fr-cfg-02), [FR-CFG-04](thermodynamics_functional_requirements_v0_1.md#fr-cfg-04), [FR-EQL-02](thermodynamics_functional_requirements_v0_1.md#fr-eql-02), [FR-PRP-07](thermodynamics_functional_requirements_v0_1.md#fr-prp-07), [FR-CHM-05](thermodynamics_functional_requirements_v0_1.md#fr-chm-05), [FR-FLW-04](thermodynamics_functional_requirements_v0_1.md#fr-flw-04), [FR-FLW-05](thermodynamics_functional_requirements_v0_1.md#fr-flw-05), [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03). **Global rules:** SI-14, SI-15. **Worked examples:** SE-04, SE-05.

<a id="ic-22"></a>
#### IC-22. Physical eligibility and default policy

**Owner:** P03. **Kind:** definition. **Step-5 product:** A06.

The reusable package's admitted domains, constituents and physical transformations and the local specializations it allows. This declares possibility, not actual phase presence or searched scope.

**Identity:** Policy revision with constituent/domain eligibility and default semantics; changes to physical restrictions are not merely numerical-policy revisions.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Admitted domains and constituents | 1..* | Eligible constituent-domain combinations, including candidate solids and fluid categories. | Representation/domain references. |
| Default transformation policy | 1 | Phase transfer, chemistry, frozen material and permitted local restrictions. | Physical-policy meanings. |
| Allowed local overlays | 0..* | Specializations a location/request may apply and conflict-resolution rules. | No undocumented evaluation-order precedence. |
| Scope limitations | 1 | Known physical/model limits and explicitly unassessed domains. | Not a numerical phase-count guarantee. |

**Permitted variants:** Nonreactive vapor/liquid package. Reactive aqueous/mineral package. Empirical carrier with equilibrium-inactive solids.

**Invariants:** Permitted phases, structurally allocated slots and actually examined candidates are different information. A physical solid suppression must survive into the effective problem and result lineage.

**Lifecycle and mutability:** Default revision is reusable; local restrictions are retained in the binding/problem rather than altering the original by side effect.

**Related concepts:** [IC-01](#ic-01), [IC-03](#ic-03), [IC-06](#ic-06), [IC-15](#ic-15), [IC-45](#ic-45).

**Valid interpretation:** A package admits precipitation, but a deliberately suppressed-solid case records a local restriction.

**Invalid interpretation:** A fallback drops the second liquid but reports the original unrestricted phase scope.

**Requirement basis:** [FR-CFG-05](thermodynamics_functional_requirements_v0_1.md#fr-cfg-05), [FR-CFG-07](thermodynamics_functional_requirements_v0_1.md#fr-cfg-07), [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08). **Global rules:** SI-09, SI-10. **Worked examples:** SE-15.

<a id="ic-23"></a>
#### IC-23. Method and convention compatibility assessment

**Owner:** P03. **Kind:** assessment. **Step-5 product:** A06.

The bounded assessment that selected phase, caloric, chemical and effective-property methods can be combined for a declared use.

**Identity:** Assessment identity captures exact subjects, intended operations, evidence and evaluator/review revision.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Assessed subjects | 1..* | Methods, data selections, conventions and material scope being combined. | Exact referenced revisions or immutable exported descriptors. |
| Required relationships | 1..* | Conditions such as compatible liquid standard state, complete caloric contribution or consistent species mapping. | Explicit relation and intended operation, not generic API compatibility. |
| Per-relationship conclusion | 1..* | Compatible under stated assumptions, incompatible, conditionally usable or unassessed, with reasons. | Unknown cannot pass a mandatory condition. |
| Evidence and restrictions | 0..* | Source data, mathematical checks or prior qualification supporting conclusions. | Validity envelope and reference evidence. |

**Permitted variants:** Whole-provider coherence assessed as a bundle. Cross-method composition with explicit reconciliation.

**Invariants:** An integration adapter that maps fields does not by itself establish scientific consistency. A supported phase split alone cannot certify a heater or reactive energy balance.

**Lifecycle and mutability:** New subject revisions require a new or explicitly reassessed conclusion. Assessment is not a mutable flag on a method name.

**Related concepts:** [IC-03](#ic-03), [IC-10](#ic-10), [IC-14](#ic-14), [IC-15](#ic-15), [IC-18](#ic-18), [IC-21](#ic-21).

**Valid interpretation:** An activity model plus vapor method is qualified only with the necessary vapor-pressure/Henry and caloric conventions.

**Invalid interpretation:** A numeric activity vector is deemed compatible with any enthalpy function solely because the units can be converted.

**Requirement basis:** [FR-CFG-02](thermodynamics_functional_requirements_v0_1.md#fr-cfg-02).

<a id="ic-24"></a>
#### IC-24. Operation-specific readiness assessment

**Owner:** P03. **Kind:** assessment. **Step-5 product:** A10.

A dependency-aware account of whether a particular physical request can be attempted and what is missing. It aggregates facts from their owners without collapsing them to one package-wide Boolean.

**Identity:** Assessment identity captures package revision, request descriptor, intended use and attestation/dependency snapshots at assessment time.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Package and demand descriptor | 1 | The configured system and immutable description of the demanded operation/output/formulation. | Exact package and exported operation meaning, not a dependency on a live execution object. |
| Prerequisite categories | 1..* | Representation, parameters, compatible methods, calorics, transport, derivatives, dependency availability, algorithm, initializer and process-boundary needs. | Each category has satisfied/unsatisfied/unassessed/inapplicable status and reasons. |
| Supporting attestations | 0..* | Provider capability, specification and process facts delivered by coordinator in this contract's vocabulary. | Provenance and exact subject/build identity; not universal upstream claims. |
| Effective result and limitations | 1 | Eligible to attempt specified operation, blocked or unassessed, including permitted limited operations. | Numerical success and independent validation are separate. |
| Dependency fingerprint | 1 | Relevant exact revisions/attestations or conservative captured context for reassessment. | No persistent native handle as dependency identity. |

**Permitted variants:** Ready for density only. PH blocked by missing ideal caloric contribution. Equation contribution ready with intentionally unresolved process unknowns.

**Invariants:** Readiness to attempt is not a promise of convergence. Initializer failure is not proof of physical infeasibility. Unavailable optional report data do not block a separate complete primary operation.

**Lifecycle and mutability:** Recompute or reassess when a relevant fact changes; expired evidence cannot be made current by relabeling.

**Related concepts:** [IC-20](#ic-20).

**Valid interpretation:** Compressor readiness checks the ideal PS reference and the actual outlet operation separately.

**Invalid interpretation:** Model construction succeeded, so every downstream unit mode is marked ready.

**Requirement basis:** [FR-GOV-01](thermodynamics_functional_requirements_v0_1.md#fr-gov-01), [FR-GOV-03](thermodynamics_functional_requirements_v0_1.md#fr-gov-03), [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03), [FR-CFG-04](thermodynamics_functional_requirements_v0_1.md#fr-cfg-04), [FR-RUN-01](thermodynamics_functional_requirements_v0_1.md#fr-run-01). **Global rules:** SI-13, SI-28. **Worked examples:** SE-07, SE-14.

### P04 — Material states, quantities and local descriptions

<a id="ic-25"></a>
#### IC-25. Local material account

**Owner:** P04. **Kind:** description. **Step-5 product:** A08.

The scope of material quantities being described at a location and snapshot: one flowing material, an inventory, an intensive-only description, or a declared reference-amount calculation. Alternative representations of it are not additional material.

**Identity:** Account identity is local to the host process location and context. Snapshot/revision distinguishes successive descriptions. Cross-unit material transfer relates accounts; it does not require one global conserved object ID.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Account purpose | 1 | Flowing material, physical inventory, intensive-only state, or calculation-reference amount. | No reference mole amount interpreted as actual equipment inventory. |
| Location/context | 1 | Host location, process scope and optional physical time; reference may be unresolved in a draft only. | Opaque host location identity, not graphical coordinates. |
| Authoritative material representation | 1 | Canonical accounting description for this snapshot; other descriptions reference the same account. | Representation revision. |
| Totals and partitions | 0..* | Known amount/rate totals and disjoint domain/phase portions; undefined amounts remain explicit. | Mass/moles/rates/loading totals with stated accounting basis. |
| Alternative views | 0..* | Apparent/true/lumped/reporting views and their mappings to the one account. | Non-additive view role; no duplicate inventory. |
| Partition/reconciliation rules | 1 | Which portions sum to a total and which overlap or merely report it. | Only disjoint portions on compatible bases are summed. |

**Permitted variants:** Stream amount-rate account. Closed vessel inventory. Unit-mole chemical calculation basis without actual plant inventory.

**Invariants:** A material account has one declared total-accounting scope per snapshot, even when many views exist. Sum leaf portions once; an overall-liquid aggregate is not an extra leaf. Intensive-only descriptions do not fabricate total energy, mass or flow.

**Lifecycle and mutability:** Input/candidate snapshots are immutable descriptions; P10 separately establishes currentness. Transfers and time integration generate new snapshots, not silent representation duplication.

**Related concepts:** [IC-03](#ic-03), [IC-05](#ic-05), [IC-26](#ic-26), [IC-27](#ic-27), [IC-29](#ic-29), [IC-31](#ic-31), [IC-47](#ic-47).

**Valid interpretation:** Liquid ions and an apparent salt report share one account; the report is excluded from additive totals.

**Invalid interpretation:** Adding full mixture mass to phase masses and reporting twice the inventory.

**Requirement basis:** [FR-MAT-02](thermodynamics_functional_requirements_v0_1.md#fr-mat-02), [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03), [FR-MAT-04](thermodynamics_functional_requirements_v0_1.md#fr-mat-04), [FR-STA-01](thermodynamics_functional_requirements_v0_1.md#fr-sta-01), [FR-CHM-06](thermodynamics_functional_requirements_v0_1.md#fr-chm-06), [FR-FLW-02](thermodynamics_functional_requirements_v0_1.md#fr-flw-02), [FR-EXT-03](thermodynamics_functional_requirements_v0_1.md#fr-ext-03). **Global rules:** SI-06, SI-07, SI-08. **Worked examples:** SE-01, SE-02, SE-03, SE-07, SE-08, SE-09, SE-10.

<a id="ic-26"></a>
#### IC-26. Quantity/value assertion

**Owner:** P04. **Kind:** assertion. **Step-5 product:** A08.

A quantity with its physical meaning, basis, scope, value role and availability. One numeric field plus a unit string is insufficient.

**Identity:** Assertion identity includes subject/account/snapshot, quantity meaning, role and source. Equivalent display conversions reference the same physical assertion rather than a new experiment.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Quantity and subject meaning | 1 | Named observable or amount, exact material/domain/component/pair scope and semantic definition reference or exported descriptor. | Scalar/vector/tensor shape and labeled axes; property definition remains P05-owned. |
| Value availability and payload | 1 | Known value, bounded interval, unresolved/deferred, undefined, unavailable, failed or withheld; payload only under an allowed interpretation. | Known zero differs from absent data; NaN is not a semantic status. |
| Unit and basis | 1 | Dimension, amount normalization, denominator, scale/offset conversion and applicable reference conditions. | Absolute temperature versus temperature difference; mass/molar/specific/total explicit. |
| Role and authority source | 1 | Fixed constraint, observation, guess, inherited default accepted as choice, supplied allocation or calculated value. | Role does not follow from numeric magnitude or whether a cell is populated. |
| Conditions and conventions | 0..* | Actual evaluation/input conditions, chemical/caloric convention descriptor, domain restrictions and uncertainty evidence where relevant. | Reference conditions are not actual conditions. |
| Origin and dependency references | 1..* | User/process assertion, measured evidence, derivation or producing calculation plus captured revisions. | Lineage; no automatic acceptance or currentness. |

**Permitted variants:** A supplied 350 K constraint. An independently measured value with uncertainty. A 345 K numerical guess. Known zero mass flow retaining a separate intended composition.

**Invariants:** Undefined specific enthalpy of absent material is not a successful zero enthalpy. Unknown is allowed where the description is partial; required unknown values block completion, not storage. Units conversion preserves physical intent; basis/reference/model transformation can require more information. Uncertainty, numerical error and model discrepancy remain distinct qualifications.

**Lifecycle and mutability:** New assertion on semantic value/role change; evaluations create derived assertions, not overwrite the originals.

**Related concepts:** [IC-10](#ic-10).

**Valid interpretation:** A failed viscosity result carries failed/unavailable status and cannot satisfy a numeric-zero query.

**Invalid interpretation:** A previous solved temperature is silently reclassified as the user's fixed outlet specification.

**Requirement basis:** [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03), [FR-MAT-05](thermodynamics_functional_requirements_v0_1.md#fr-mat-05), [FR-MAT-06](thermodynamics_functional_requirements_v0_1.md#fr-mat-06), [FR-STA-04](thermodynamics_functional_requirements_v0_1.md#fr-sta-04), [FR-STA-07](thermodynamics_functional_requirements_v0_1.md#fr-sta-07), [FR-STA-08](thermodynamics_functional_requirements_v0_1.md#fr-sta-08), [FR-EXT-03](thermodynamics_functional_requirements_v0_1.md#fr-ext-03). **Global rules:** SI-03, SI-04, SI-07. **Worked examples:** SE-01, SE-06, SE-07, SE-08, SE-09, SE-10.

<a id="ic-27"></a>
#### IC-27. Composition description

**Owner:** P04. **Kind:** description. **Step-5 product:** A08.

A set of constituent quantities, fractions or ratios for one account or portion, with explicit basis and independent versus derived values.

**Identity:** Identity fixes account/portion, snapshot, representation/coordinate revision and assertion role. A permutation of storage does not change physical composition.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Account and scope | 1 | Whole material or identified disjoint portion/phase; alternative reporting view explicitly labeled. | Exactly one account scope per description. |
| Coordinate system | 1 | Identity/order/chart and normalization denominator of the entries. | Mass/molar/rate/concentration/ratio basis. |
| Entries and role | 1..* | Known or explicitly unresolved values associated with coordinates; specified/calculated/guess distinctions retained. | Uses quantity assertion semantics; no default missing=zero. |
| Total/normalization context | 0..1 | Known positive denominator or separate total; may be absent for an intensive-only normalized composition specification. | Do not infer a composition from zero amounts by division. |
| Relationship to other views | 0..* | Map, constraint or chemical reconciliation connecting this description to another of the same material. | No new additive inventory. |

**Permitted variants:** Normalized fraction input with unknown total. Component flows from which total and fractions are derived. Partially specified equation-model composition.

**Invariants:** Physical accepted fractions are nonnegative and obey the declared normalization within predeclared tolerances. Amounts may be all zero while intended feed composition is separately known. A full replacement, a partial patch and total rescaling are different change intents.

**Lifecycle and mutability:** Replace/patch/rescale yields a new proposal or snapshot; preserve old role and coordinate lineage.

**Related concepts:** [IC-04](#ic-04), [IC-07](#ic-07), [IC-25](#ic-25), [IC-26](#ic-26), [IC-47](#ic-47).

**Valid interpretation:** Changing A from 2 to 4 kg/s while preserving B=3 kg/s yields total 7 and fractions 4/7,3/7.

**Invalid interpretation:** Keeping total fixed at 5 while also holding A=4,B=3 without reporting conflicting constraints.

**Requirement basis:** [FR-MAT-04](thermodynamics_functional_requirements_v0_1.md#fr-mat-04), [FR-MAT-05](thermodynamics_functional_requirements_v0_1.md#fr-mat-05), [FR-MAT-06](thermodynamics_functional_requirements_v0_1.md#fr-mat-06), [FR-STA-07](thermodynamics_functional_requirements_v0_1.md#fr-sta-07), [FR-FLW-02](thermodynamics_functional_requirements_v0_1.md#fr-flw-02), [FR-LIF-02](thermodynamics_functional_requirements_v0_1.md#fr-lif-02). **Global rules:** SI-06, SI-08. **Worked examples:** SE-02, SE-03, SE-10.

<a id="ic-28"></a>
#### IC-28. Local state snapshot

**Owner:** P04. **Kind:** description. **Step-5 product:** A08.

A time/revision-specific description of material conditions and local domains, possibly partial and not necessarily at phase or chemical equilibrium.

**Identity:** Snapshot identity fixes account(s), input/context revision and source. Phase instance identities are snapshot-local unless correspondence evidence relates them.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Material account | 1 | The local physical or reference-amount scope being described. | Flow/inventory/intensive/reference purpose explicit. |
| State-value assertions | 0..* | Temperature, pressure, volume, energy and other known or unresolved coordinates with their roles. | No universal two-independent-scalars rule. |
| Composition descriptions | 1..* | Whole and/or domain-local material descriptions; completeness can be partial. | Each description has explicit normalization/account relation. |
| Domain/phase portions | 0..* | Present domains plus separately labeled incipient or numerical entries if retained. | Presence and quantities explicit; no imposed equilibrium merely from co-location. |
| Additional material attributes | 0..* | Distribution or surface realizations appropriate to the representation. | Defined attribute and quantity bases. |
| Configuration/time references | 1 | Captured effective material/configuration/binding/time context or explicit unresolved draft references. | Physical time distinct from wall-clock solve timestamps. |
| Declared state interpretation | 1 | Equilibrated candidate, supplied nonequilibrium, constrained phase, incomplete input, etc.; this is a claim requiring separate evidence. | No accepted/current Boolean embedded as sole quality authority. |

**Permitted variants:** Single bulk state. Multiphase equilibrium candidate. Separate bulk states plus interface relations. Inventory U,V specification with unresolved T,P.

**Invariants:** One material account can contain distinct phase temperatures for a declared nonequilibrium formulation. A populated state is not automatically a completed or accepted state. No state needs a graphical stream or fictitious flow.

**Lifecycle and mutability:** Create captured descriptions; append derived snapshots; acceptance and publication refer to them separately.

**Related concepts:** [IC-25](#ic-25), [IC-26](#ic-26), [IC-27](#ic-27), [IC-29](#ic-29), [IC-32](#ic-32), [IC-33](#ic-33).

**Valid interpretation:** A column stage retains guessed temperature, fixed pressure and unresolved composition constraints without claiming a solved stage.

**Invalid interpretation:** An interface-property call homogenizes two caller-owned bulk states.

**Requirement basis:** [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03), [FR-MAT-04](thermodynamics_functional_requirements_v0_1.md#fr-mat-04), [FR-STA-01](thermodynamics_functional_requirements_v0_1.md#fr-sta-01), [FR-STA-04](thermodynamics_functional_requirements_v0_1.md#fr-sta-04), [FR-PRP-01](thermodynamics_functional_requirements_v0_1.md#fr-prp-01), [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08), [FR-EXT-03](thermodynamics_functional_requirements_v0_1.md#fr-ext-03). **Global rules:** SI-08. **Worked examples:** SE-01, SE-02, SE-10, SE-12, SE-15.

<a id="ic-29"></a>
#### IC-29. Domain portion and phase instance

**Owner:** P04. **Kind:** description. **Step-5 product:** A08.

A physical portion of one local material account or a separately labeled incipient/numerical domain entry. Its identity is not a phase enum index or product port.

**Identity:** Local snapshot plus domain-entry identity. Physical correspondence across snapshots is a separate IC-30 assertion; unique continuation is not assumed.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Parent account and snapshot scope | 1 | Exactly one accounting/snapshot scope for the portion. | Disjoint-partition membership or non-additive role declared. |
| Domain definition and candidate origin | 1 | Domain kind plus applicable declared candidate/policy origin. | Liquid category does not choose liquid instance identity. |
| Presence and information role | 1 | Present physical, absent physical, incipient limit, numerical placeholder or reporting aggregate prohibited here. | Presence evidence/tolerance retained; no rounding destroys conservation. |
| Quantity and composition | 0..* | Amount/fraction and composition when physically meaningful; absent phase may retain incipient values only with that role. | Fraction denominator and amount basis explicit. |
| Local intensive assertions | 0..* | Actual or requested phase T,P,density and other local coordinates. | May differ across nonequilibrium portions. |
| Phase-specific species/structure attributes | 0..* | Solid form, solution/site or relevant domain descriptors when needed. | No phase-specific property inferred from constituent name alone. |

**Permitted variants:** Two liquid instances sharing one liquid method. Unreactive solid portion. Zero-amount incipient vapor carrying continuation information.

**Invariants:** Aggregate mixture and total-liquid views are IC-31, not additional physical phases. Identical numerical entries need not be two physical phases; amount and role determine interpretation. A stable-phase claim requires evidence, not just a vapor/liquid label.

**Lifecycle and mutability:** Within-snapshot identity fixed; transitions may create/disappear/merge portions and generate explicit correspondence information.

**Related concepts:** [IC-01](#ic-01), [IC-06](#ic-06), [IC-25](#ic-25), [IC-26](#ic-26), [IC-27](#ic-27), [IC-28](#ic-28), [IC-32](#ic-32), [IC-33](#ic-33).

**Valid interpretation:** A zero-amount liquid candidate retains its incipient composition but has no physical outlet mass.

**Invalid interpretation:** A separator creates positive product flow because the provider returned a liquid slot.

**Requirement basis:** [FR-CFG-07](thermodynamics_functional_requirements_v0_1.md#fr-cfg-07), [FR-STA-05](thermodynamics_functional_requirements_v0_1.md#fr-sta-05), [FR-STA-06](thermodynamics_functional_requirements_v0_1.md#fr-sta-06), [FR-EQL-04](thermodynamics_functional_requirements_v0_1.md#fr-eql-04), [FR-EQL-06](thermodynamics_functional_requirements_v0_1.md#fr-eql-06), [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08), [FR-PRP-03](thermodynamics_functional_requirements_v0_1.md#fr-prp-03), [FR-FLW-03](thermodynamics_functional_requirements_v0_1.md#fr-flw-03), [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08). **Global rules:** SI-06, SI-09. **Worked examples:** SE-02, SE-15.

<a id="ic-30"></a>
#### IC-30. Cross-snapshot phase correspondence

**Owner:** P04. **Kind:** assessment. **Step-5 product:** A08.

An explicit relation between phase/domain entries in two snapshots, used for comparison or continuation without confusing identity with ordering.

**Identity:** Correspondence identity fixes source snapshot, target snapshot, relation policy/version and evidence. It never globally renames historical phase entries.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Source entries | 0..* | Prior phase/domain entries, including none for appearance. | Exact snapshot-local identities. |
| Target entries | 0..* | New entries, including none for disappearance. | Exact snapshot-local identities. |
| Relation kind | 1 | Continuation, appearance, disappearance, split, merge, relabel/equivalence, ambiguous or unassessed. | Many-to-many relation allowed; both sides cannot be empty. |
| Evidence and admissibility | 1 | Representation/domain compatibility, compositions/properties used, phase-policy compatibility and thresholds. | Declared criteria; equal density is not sufficient identity evidence. |
| Confidence/ambiguity statement | 1 | Unique under declared criteria, alternatives remain, or no assessed match; numeric score only if its definition exists. | No fabricated probabilistic confidence. |

**Permitted variants:** One-to-one continuation. Two-to-one coalescence. Ambiguous matching after a branch change.

**Invariants:** Phase ordering invariance is required, but unique continuity at coalescence is not. Product-port assignment is a different relation. A composition basis or constituent-order change must be reconciled before comparing compositions.

**Lifecycle and mutability:** Append assessment between immutable snapshots; later policy revisions produce another assessment, not changed old identities.

**Related concepts:** [IC-29](#ic-29).

**Valid interpretation:** When two liquids coalesce, both predecessor entries relate to one successor without pretending either survived uniquely.

**Invalid interpretation:** Sorting by density renames the phases when densities cross and corrupts a history comparison.

**Requirement basis:** [FR-STA-05](thermodynamics_functional_requirements_v0_1.md#fr-sta-05), [FR-STA-06](thermodynamics_functional_requirements_v0_1.md#fr-sta-06), [FR-EQL-07](thermodynamics_functional_requirements_v0_1.md#fr-eql-07), [FR-FLW-03](thermodynamics_functional_requirements_v0_1.md#fr-flw-03), [FR-EXT-04](thermodynamics_functional_requirements_v0_1.md#fr-ext-04). **Global rules:** SI-09. **Worked examples:** SE-02, SE-15.

<a id="ic-31"></a>
#### IC-31. Aggregate and reporting view

**Owner:** P04. **Kind:** description. **Step-5 product:** A08.

A derived perspective on existing material, such as overall liquid, dry-gas ratio, standard volume or apparent composition; it is not another physical phase or inventory.

**Identity:** View identity fixes source snapshots/portions, quantity meaning, basis, transformation/closure and any evaluation dependencies.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Sources and selection | 1..* | Accounts/portions or property assertions used once according to the view rule. | Disjoint or overlapping selection explicit. |
| Reporting meaning and basis | 1 | Observable, wet/dry/solvent/standard-condition convention and denominator. | Reference T,P and volume convention when required. |
| Derivation rule | 1 | Algebraic sum/ratio, representation map, effective closure or additional property evaluation. | Unit conversion distinguished from reference-state/model calculation. |
| Additional evaluated evidence | 0..* | Needed density, standard-state or effective property with captured conventions and source. | Missing required evidence leaves the view incomplete. |
| Additivity declaration | 1 | Non-additive alias/view, or a sum over explicitly disjoint physical portions. | A view cannot be counted again in its parent account. |

**Permitted variants:** Pure algebraic composition projection. Named effective mixture closure. A hypothetical reference-condition property evaluation.

**Invariants:** Overall-liquid aggregation does not add another liquid phase. Standard volume cannot be obtained without the stated reference conditions and required model. A dry-basis report does not remove actual water from the process material.

**Lifecycle and mutability:** Recompute on changed dependencies; optional view failure leaves an independent accepted primary result intact.

**Related concepts:** [IC-07](#ic-07), [IC-25](#ic-25), [IC-26](#ic-26), [IC-29](#ic-29).

**Valid interpretation:** A report shows dry-gas composition while the source account retains its water amount.

**Invalid interpretation:** An unavailable reporting density silently changes the material to a different model.

**Requirement basis:** [FR-STA-05](thermodynamics_functional_requirements_v0_1.md#fr-sta-05), [FR-STA-08](thermodynamics_functional_requirements_v0_1.md#fr-sta-08), [FR-PRP-04](thermodynamics_functional_requirements_v0_1.md#fr-prp-04). **Global rules:** SI-06, SI-19. **Worked examples:** SE-02.

<a id="ic-32"></a>
#### IC-32. Local distributed-attribute realization

**Owner:** P04. **Kind:** description. **Step-5 product:** A08.

A distribution or moment payload attached to a specific amount/portion of material under a defined weighting and support convention.

**Identity:** Realization identity fixes definition, local association, snapshot, payload and source/reduction lineage.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Attribute definition | 1 | Support, measure, weighting and reduction semantics. | Exact definition revision. |
| Associated material | 1 | Account or portion and actual amount used to weight or convert the attribute. | Amount basis must match the weighting measure or have an explicit conversion. |
| Payload and completeness | 1 | Binned weights, density function or finite moments with explicit missing information. | Axes, bins and normalization; moments retain their units. |
| Reduction/transfer lineage | 0..* | Rebinning, mixing, reduction or selective transfer applied to the original distribution. | Information loss and conservation claim explicit. |

**Permitted variants:** Full normalized discrete distribution and amount. Non-normalized amount-per-bin distribution. Moment-only reduced representation.

**Invariants:** One scalar mean cannot be relabeled a full distribution. Mixing incompatible bins or weighting measures requires a declared transformation. Selective transfer must state how attribute and material amount change together.

**Lifecycle and mutability:** New snapshot on transfer/mix; preserve original payload when available and record all reductions.

**Related concepts:** [IC-07](#ic-07), [IC-08](#ic-08), [IC-10](#ic-10), [IC-25](#ic-25), [IC-26](#ic-26), [IC-29](#ic-29).

**Valid interpretation:** 1 kg with mass-bin weights [0.8,0.2] plus 3 kg with [0.2,0.8] gives [0.35,0.65] over identical bins.

**Invalid interpretation:** Equal-weight averaging gives [0.5,0.5] and is mislabeled mass-conserving mixing.

**Requirement basis:** [FR-EXT-02](thermodynamics_functional_requirements_v0_1.md#fr-ext-02). **Global rules:** SI-27. **Worked examples:** SE-08.

<a id="ic-33"></a>
#### IC-33. Local surface/loading realization

**Owner:** P04. **Kind:** description. **Step-5 product:** A08.

The actual nonbulk stored material and loading at a location, separate from bulk material and the process-owned denominator quantity.

**Identity:** Realization fixes domain/account, basis definition, snapshot and actual supporting denominator context.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Basis definition | 1 | Loading/occupancy and amount-conversion meaning. | Area, dry mass, sites or other explicit denominator. |
| Local account and domain | 1 | Surface/nonbulk portion being described. | Separate accounting portion, not fictitious flowing liquid. |
| Denominator and loading | 1..* | Known or unresolved actual denominator and supported loading quantities. | Units and dry/wet or excess/total convention retained. |
| Stored material and site account | 0..* | Derived or supplied species amounts and occupied/vacant sites where meaningful. | Justified conserved basis and occupancy checks. |
| Exchange links | 0..* | Transfers relating loading changes to bulk or reservoir quantities. | Signed amount/rate and interval explicit. |

**Permitted variants:** Mass-normalized adsorption. Site-occupancy inventory. Reference-only interface description without a physical stored amount.

**Invariants:** Unknown denominator permits an intensive loading description but not a unique total amount. Bulk and adsorbed material are disjoint portions only under an explicit accounting convention.

**Lifecycle and mutability:** Update through new transfer/state proposals; site capacity and equilibrium support are separately qualified.

**Related concepts:** [IC-05](#ic-05), [IC-09](#ic-09), [IC-25](#ic-25), [IC-26](#ic-26), [IC-27](#ic-27), [IC-29](#ic-29), [IC-71](#ic-71).

**Valid interpretation:** An authorized 0.1 mol transfer from bulk adds 0.1 mol to the supported total adsorbed amount.

**Invalid interpretation:** Increasing loading without decreasing any source or declaring an external exchange.

**Requirement basis:** [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01). **Global rules:** SI-27. **Worked examples:** SE-09.

<a id="ic-71"></a>
#### IC-71. Physical transfer and exchange realization

**Owner:** P04. **Kind:** description. **Step-5 product:** A08.

The actual or specified material/energy transfer between accounts or with an external reservoir on a stated amount/rate/time basis. It is not an additional inventory.

**Identity:** Transfer identity fixes source/sink contexts, quantity meaning, temporal support and producing problem/input assertion.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Source and sink | 1 | Both material accounts or an explicitly identified external reservoir/boundary. | Direction convention, not negative physical composition. |
| Transferred quantities | 1..* | Component/conserved material quantities and heat/work/energy contributions where applicable. | Instantaneous rate versus integrated amount and units explicit. |
| Temporal basis | 1 | Steady rate, time-dependent rate at a stated time, or finite-interval amount; reference calculation amount explicitly labeled. | No implicit one-second conversion of inventory to flow. |
| Energy/reference convention | 0..1 | Required for energy transfer: carried enthalpy/internal-energy or genuine heat/work and formation convention. | No double counting reference correction as physical heat. |
| Permission and origin | 1 | Specified transfer or actual result under the declared unit/chemical authority. | Captured problem/source evidence, including bounds where imposed. |

**Permitted variants:** Steady flow across a unit boundary. Finite reservoir addition. Bulk-to-surface amount transfer. Heat-only transfer with no species exchange.

**Invariants:** Equal/opposite internal transfer enters account balances once on each side with a declared sign. Composition fractions remain nonnegative when transfer direction reverses. A unit-mole equilibrium calculation becomes a physical rate only through an explicit valid scaling context.

**Lifecycle and mutability:** Record specified or solved transfer per captured context; process integration/time advancement creates new account snapshots.

**Related concepts:** [IC-05](#ic-05), [IC-21](#ic-21), [IC-25](#ic-25), [IC-26](#ic-26), [IC-35](#ic-35), [IC-46](#ic-46).

**Valid interpretation:** An explicit titrant amount is included in both chemical conservation and the appropriate energy balance.

**Invalid interpretation:** A speciation calculation silently creates a feed stream to satisfy pH.

**Requirement basis:** [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03), [FR-STA-07](thermodynamics_functional_requirements_v0_1.md#fr-sta-07), [FR-CHM-05](thermodynamics_functional_requirements_v0_1.md#fr-chm-05), [FR-CHM-07](thermodynamics_functional_requirements_v0_1.md#fr-chm-07), [FR-FLW-02](thermodynamics_functional_requirements_v0_1.md#fr-flw-02), [FR-FLW-03](thermodynamics_functional_requirements_v0_1.md#fr-flw-03), [FR-FLW-04](thermodynamics_functional_requirements_v0_1.md#fr-flw-04), [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03), [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01). **Global rules:** SI-07, SI-15, SI-16. **Worked examples:** SE-03, SE-04, SE-05, SE-09, SE-10.

### P05 — Thermodynamic problems and operation contracts

<a id="ic-34"></a>
#### IC-34. Operation contract

**Owner:** P05. **Kind:** definition. **Step-5 product:** A09.

The reusable meaning of a property, state-resolution, phase/chemical-equilibrium, stability, derivative or coupled-contribution operation. It is not a provider method signature.

**Identity:** Contract revision identifies admissible inputs, authority pattern, output meanings and promised completion level.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Operation purpose | 1 | Property-only, TP/PH/PS/saturation, phase/chemical equilibrium, stability, restricted branch, derivative or mathematical contribution. | Physical intent and specification meanings. |
| Input and authority obligations | 1..* | Required known quantities, permissible unresolved coordinates and transformations; reference-amount requirements when applicable. | Quantity/basis conventions, not a fixed two-scalar interface. |
| Output demands | 1..* | Observable/result semantics with mandatory, conditional or optional role and domain/pair/component scope. | Demand conditionality and completeness promise explicit. |
| Required postconditions | 1..* | What completing the operation establishes and what it does not establish. | Residual/authority/check definitions, no implied all-properties-computed promise. |
| Physical eligibility | 1 | Supported material and transformation scope as contract criteria. | Declared phase/species/branch restrictions. |

**Permitted variants:** Complete numerical state operation. A partial local equation/residual/property contribution. A fixed-state property evaluation.

**Invariants:** A phase-property operation may resolve permitted density without gaining authority to redistribute species. Equilibrium output does not imply every transport property is available. An upstream capability name is not proof the chosen adapter realizes the contract.

**Lifecycle and mutability:** Contracts are versioned definitions; submitted instances are IC-35.

**Related concepts:** [IC-22](#ic-22), [IC-36](#ic-36), [IC-37](#ic-37), [IC-38](#ic-38), [IC-39](#ic-39), [IC-40](#ic-40), [IC-41](#ic-41), [IC-42](#ic-42).

**Valid interpretation:** A PH contract requires verified target enthalpy and declared phase scope but requests viscosity only optionally.

**Invalid interpretation:** A function named flash is treated as supporting every input pair and number of phases.

**Requirement basis:** [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03), [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01), [FR-EQL-05](thermodynamics_functional_requirements_v0_1.md#fr-eql-05), [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08), [FR-PRP-01](thermodynamics_functional_requirements_v0_1.md#fr-prp-01), [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02), [FR-PRP-03](thermodynamics_functional_requirements_v0_1.md#fr-prp-03), [FR-FLW-01](thermodynamics_functional_requirements_v0_1.md#fr-flw-01). **Global rules:** SI-17, SI-18. **Worked examples:** SE-14.

<a id="ic-35"></a>
#### IC-35. Physical calculation problem

**Owner:** P05. **Kind:** problem. **Step-5 product:** A09.

The captured physical question at a location: original data, constraints, unknowns, transformation permissions, output demands and check scope. It survives algorithm changes intact.

**Identity:** Problem identity fixes captured physical inputs, package/binding revisions, conventions, local restrictions and desired completion. Attempts and initial guesses are separate identities.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Operation contract | 1 | Exact physical operation being instantiated. | Contract revision. |
| Material inputs | 1..* | Captured local state/account descriptions with required composition/amount context. | Specified versus guessed quantities retain roles. |
| Effective configuration | 1 | Base package and resolved local binding/overlay facts. | Exact references or immutable exported descriptors; not ambient state. |
| Constraints and authority | 1 | Original specified quantities, coupled unknowns and permissions. | Constraint and quantity-family semantics. |
| Restrictions and coupling | 0..* | Phase/chemistry constraints, requested branch, reservoir permissions and declared splitting approximation. | A changed physical scope creates a distinct problem. |
| Outputs and checks | 1..* | Required/optional observable/contribution demands and acceptance criteria. | Explicit scope and tolerance definitions. |
| Captured context | 1 | Model/dependency revision, process scope and original request provenance supplied as reference values. | No mutable P10 service dependency required to define meaning. |

**Permitted variants:** Standalone complete PH request. Intentionally partial equation-model state contribution. Combined reactive multiphase energy-constrained problem.

**Invariants:** Numerical retries refer to the original target; they do not redefine it to the returned value. An authorized alternative with different phases/species/model has a new problem and a relation to the original. Total U,V need justified amounts/composition for a complete inventory-state problem.

**Lifecycle and mutability:** Freeze original problem; create separately identified attempts, alternatives, candidate results and assessments.

**Related concepts:** [IC-20](#ic-20), [IC-28](#ic-28), [IC-34](#ic-34), [IC-36](#ic-36), [IC-37](#ic-37), [IC-38](#ic-38), [IC-39](#ic-39), [IC-40](#ic-40), [IC-41](#ic-41), [IC-42](#ic-42), [IC-45](#ic-45), [IC-46](#ic-46).

**Valid interpretation:** An accepted solid-suppressed alternative is linked to, but not mislabeled as, the unrestricted request.

**Invalid interpretation:** A fallback modifies the requested candidate-phase list in place then reports same-problem convergence.

**Requirement basis:** [FR-STA-02](thermodynamics_functional_requirements_v0_1.md#fr-sta-02), [FR-STA-03](thermodynamics_functional_requirements_v0_1.md#fr-sta-03), [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01), [FR-EQL-02](thermodynamics_functional_requirements_v0_1.md#fr-eql-02), [FR-EQL-03](thermodynamics_functional_requirements_v0_1.md#fr-eql-03), [FR-EQL-04](thermodynamics_functional_requirements_v0_1.md#fr-eql-04), [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08), [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03), [FR-CHM-07](thermodynamics_functional_requirements_v0_1.md#fr-chm-07), [FR-FLW-05](thermodynamics_functional_requirements_v0_1.md#fr-flw-05), [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05), [FR-EXT-03](thermodynamics_functional_requirements_v0_1.md#fr-ext-03), [FR-EXT-04](thermodynamics_functional_requirements_v0_1.md#fr-ext-04). **Global rules:** SI-10, SI-17, SI-21. **Worked examples:** SE-01, SE-05, SE-07, SE-10, SE-12, SE-15.

<a id="ic-36"></a>
#### IC-36. Specification, constraint and unknown set

**Owner:** P05. **Kind:** definition. **Step-5 product:** A09.

The joint constraints that define a local state or contribution and the quantities left unknown. It distinguishes constraint role from the mere presence of a value.

**Identity:** Identity fixes variable coordinates, constraint definitions, input roles, intended standalone/coupled completion and assessed independence.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Quantity/variable bindings | 1..* | Local observables or algebraic coordinates and their subject/domain identity. | Units, basis, composition chart and scale per coordinate. |
| Constraint statements | 0..* | Equalities, inequalities, ranges or fixed-value specifications and their origin. | Redundant observations can be checks rather than added equations. |
| Unknowns and held-fixed quantities | 1 | Explicit unknown and fixed sets, including relationship to external coupled variables. | No silent freezing from old values. |
| Completeness assessment | 1 | Intentionally partial, sufficient, under/over-specified, inconsistent or unassessed with evidence. | Counting values alone is not a rank or feasibility proof. |
| Observations/initial estimates | 0..* | Values retained with non-constraint roles unless explicitly selected. | Measurement uncertainty and guesses remain different. |

**Permitted variants:** Direct fixed independent coordinates. Balance/equilibrium constraints jointly solved with unit equations. Range-constrained or overdetermined observations requiring explicit reconciliation.

**Invariants:** Pure-fluid saturated T,P may be dependent and leave phase allocation undetermined. A partially specified coupled stage is a legitimate description, not an automatic error. Nonlinear rank/uniqueness must remain unassessed when not established.

**Lifecycle and mutability:** Original set preserved through numerical preparation; any temporary fixing is separately recorded and reversed or explicitly reconciled.

**Related concepts:** [IC-04](#ic-04), [IC-26](#ic-26), [IC-38](#ic-38).

**Valid interpretation:** A temperature measurement can be a validation observation while the process solver determines temperature from PH.

**Invalid interpretation:** A guess is left fixed after initialization and silently adds an extra physical constraint.

**Requirement basis:** [FR-STA-02](thermodynamics_functional_requirements_v0_1.md#fr-sta-02), [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01), [FR-EQL-02](thermodynamics_functional_requirements_v0_1.md#fr-eql-02), [FR-EQL-03](thermodynamics_functional_requirements_v0_1.md#fr-eql-03), [FR-EQL-04](thermodynamics_functional_requirements_v0_1.md#fr-eql-04), [FR-CHM-02](thermodynamics_functional_requirements_v0_1.md#fr-chm-02), [FR-FLW-07](thermodynamics_functional_requirements_v0_1.md#fr-flw-07). **Global rules:** SI-03, SI-20. **Worked examples:** SE-01, SE-05, SE-10, SE-12.

<a id="ic-37"></a>
#### IC-37. Calculation-authority allocation

**Owner:** P05. **Kind:** definition. **Step-5 product:** A09.

Who supplied each relevant quantity and which operation may determine or change it. Numerical ownership of a buffer is not physical authority.

**Identity:** Authority identity fixes the problem, quantity families, source assertions and permitted changes at the requested scope.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Quantity scopes | 1..* | Intensive coordinates, constituent totals, phase amounts/compositions, total amount, external exchanges or distributed attributes. | Component/domain/phase and amount basis explicit. |
| Original authority | 1..* | User specification, unit balance, external accepted allocation, chemical-system constraints or unresolved variable ownership. | Provenance of fixed quantities. |
| Permitted determinations | 1..* | Evaluate properties, resolve density/root, redistribute phases, transform species, or apply explicitly approved map. | Permissions are not inferred from provider capability. |
| Preservation obligations | 1..* | Quantities/relations that must remain unchanged, and observational checks. | Structural equality or declared numerical tolerance as appropriate. |
| Conflict assessment | 1 | Compatible, conflicting or unassessed allocation with reasons. | Unavailable evidence is not implicit consent. |

**Permitted variants:** Property-only evaluation with density/root freedom. Fixed-component phase equilibrium. Element-conserving chemical equilibrium with explicitly allowed reservoirs.

**Invariants:** Fixing an arbitrary split and requiring unrestricted equilibrium can conflict. Phase redistribution does not authorize chemical reaction. Permission to calculate does not authorize publishing a current result.

**Lifecycle and mutability:** Attached to a fixed problem; physical changes create a new authority record and problem identity.

**Related concepts:** [IC-26](#ic-26), [IC-29](#ic-29), [IC-41](#ic-41).

**Valid interpretation:** An externally supplied phase split is preserved during an enthalpy refresh.

**Invalid interpretation:** A property evaluator consumes a frozen species because the external engine equilibrates by default.

**Requirement basis:** [FR-STA-03](thermodynamics_functional_requirements_v0_1.md#fr-sta-03), [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01), [FR-PRP-01](thermodynamics_functional_requirements_v0_1.md#fr-prp-01), [FR-CHM-04](thermodynamics_functional_requirements_v0_1.md#fr-chm-04), [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08). **Global rules:** SI-17, SI-20. **Worked examples:** SE-01, SE-12, SE-15.

<a id="ic-38"></a>
#### IC-38. Property observable definition

**Owner:** P05. **Kind:** definition. **Step-5 product:** A09.

The exact property being requested: physical subject, mathematical meaning, domain, basis and conventions. Same labels or units do not establish equality of observables.

**Identity:** Observable revision fixes subject kind, definition, normalization, references and response semantics. Local numeric assertions use this meaning but do not own it.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Property meaning | 1 | Defined scalar/vector/tensor quantity, including intrinsic, additive aggregate, effective closure or diagnostic role. | Dimensions and labeled axes. |
| Subject scope | 1 | Whole material, phase/domain, component-in-phase, partial molar subject, ordered transfer pair or unordered interfacial pair. | A pair is ordered only when the observable's direction requires it. |
| Amount and denominator basis | 1 | Total, molar, mass-specific, concentration, per-carrier/site/area or other justified basis. | Extensive totals require a material amount; reference amount is explicit. |
| Reference and standard conventions | 0..* | Energy, activity, fugacity, standard volume or other required conventions. | No free-floating h,mu,pH or standard volume without its necessary context. |
| Definition/closure dependencies | 0..* | Underlying methods or effective-property assumptions and required phase inputs. | Frozen-allocation aggregate versus equilibrium response distinguished. |
| Tensor/component axes | 0..* | Required for vector/tensor transport and partial-molar quantities, including diffusion frame/reference species when applicable. | Axes, coordinate basis and constitutive sign convention. |

**Permitted variants:** Phase mass-specific enthalpy. Liquid–liquid interfacial tension for a named pair. Composition-chart diffusion matrix in a declared flux frame. Defined effective slurry viscosity.

**Invariants:** Viscosity is not a heat-transfer coefficient. A total energy and a molar energy differ even if a provider names both enthalpy. An equilibrium-response heat capacity is not replaced by a weighted frozen-phase value. A chemical potential standard convention must be compatible with the equilibrium residual in which it is used.

**Lifecycle and mutability:** Define reusable meaning; local requested scope instantiates it without changing its mathematical interpretation.

**Related concepts:** [IC-01](#ic-01), [IC-04](#ic-04), [IC-06](#ic-06), [IC-09](#ic-09), [IC-18](#ic-18), [IC-21](#ic-21).

**Valid interpretation:** Two reported Cp values have distinct definitions: fixed phase allocation and re-equilibrated total response.

**Invalid interpretation:** Both use kJ/kg/K, so they are treated as numerically interchangeable.

**Requirement basis:** [FR-EQL-04](thermodynamics_functional_requirements_v0_1.md#fr-eql-04), [FR-EQL-05](thermodynamics_functional_requirements_v0_1.md#fr-eql-05), [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02), [FR-PRP-03](thermodynamics_functional_requirements_v0_1.md#fr-prp-03), [FR-PRP-04](thermodynamics_functional_requirements_v0_1.md#fr-prp-04), [FR-PRP-05](thermodynamics_functional_requirements_v0_1.md#fr-prp-05). **Global rules:** SI-04, SI-18, SI-19. **Worked examples:** SE-06, SE-07.

<a id="ic-39"></a>
#### IC-39. Derivative and solved-response specification

**Owner:** P05. **Kind:** definition. **Step-5 product:** A09.

The meaning of a derivative, including output/input coordinates, constraints held fixed, allowed phase/chemical response, derivative order and coordinate transformations.

**Identity:** Identity is the complete derivative experiment, not merely a variable pair. Changes of independent chart or equilibrium response create a different observable.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Output observable and input chart | 1 | What is differentiated and with respect to which independent state/parameter/amount coordinates. | Output units divided by input units for first derivatives; explicit tensor axes for higher order. |
| Held-fixed conditions | 1..* | Pressure, composition, amount, phase fractions, parameters or other constraints held fixed in this derivative. | Exact quantity/domain conventions. |
| Response mode | 1 | Local homogeneous, fixed allocation, saturation path, constrained re-equilibrium, chemical response, parameter sensitivity or declared other path. | No inference from a generic automatic-differentiation flag. |
| Order and orientation | 1 | First/higher/mixed order, full or directional derivative, tensor index order and normalization. | Composition chart and direction must obey the specified constraints. |
| Branch and validity demands | 1 | Active branch, one-sided/two-sided expectation, transition restrictions and conditioning checks requested. | Nonexistence, ambiguity and not-assessed smoothness remain separate. |
| Coordinate/chain-rule relation | 0..* | Transformations needed to compare derivative coordinates, including parameter reference changes. | Explicit Jacobian/chart conventions, not just unit conversion. |

**Permitted variants:** Local d h/dT at fixed P,x. PH-flash temperature sensitivity to total feed composition in an N−1 chart. Second derivative or directional sensitivity with explicit available evidence.

**Invariants:** N normalized fractions do not define N independent directions without a constraint convention. Derivative of an equation residual differs from total sensitivity of a solved constrained state. The method used to obtain a derivative belongs to execution evidence; it does not define what derivative was requested.

**Lifecycle and mutability:** Frozen request semantics; each computed tensor is a candidate carrying actual method, branch and verification evidence.

**Related concepts:** [IC-04](#ic-04), [IC-07](#ic-07), [IC-12](#ic-12), [IC-36](#ic-36), [IC-38](#ic-38), [IC-41](#ic-41), [IC-42](#ic-42).

**Valid interpretation:** An equilibrium enthalpy response explicitly allows phase fractions and compositions to change with temperature.

**Invalid interpretation:** A homogeneous derivative is returned for a perturbation that crosses a phase transition without qualification.

**Requirement basis:** [FR-PRP-05](thermodynamics_functional_requirements_v0_1.md#fr-prp-05), [FR-PRP-06](thermodynamics_functional_requirements_v0_1.md#fr-prp-06). **Global rules:** SI-05, SI-19. **Worked examples:** SE-06, SE-12.

<a id="ic-40"></a>
#### IC-40. Mathematical contribution description

**Owner:** P05. **Kind:** definition. **Step-5 product:** A09.

The local thermodynamic relationships supplied to a coupled unit/process formulation without imposing one solver, symbolic language or global equation graph.

**Identity:** Contribution revision fixes local variable roles, relationships, external mappings and supported value/residual/equation/derivative semantics.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Local variables and bindings | 1..* | Local coordinates mapped explicitly to host problem variables or fixed parameters. | Units, bases, composition charts and scale meanings. |
| Relationships and contribution kind | 1..* | Values, residuals, equations, inequalities or qualified callbacks actually provided. | Equation semantics and residual normalization; no required syntax/IR. |
| Physical responsibility boundary | 1 | What the contribution enforces versus unit/host balances and unknowns it only consumes. | No duplicate independent enforcement of already fixed quantities. |
| Derivative dependencies and pattern | 0..* | Actually available Jacobian/Hessian/directional information and structural versus state-observed dependence. | A numerical zero is not proof of structural independence. |
| Bounds and scaling descriptors | 0..* | Physical admissibility, numerical search bounds and scaling as separately identified facts. | Original physical units recoverable; scaling does not redefine a constraint. |
| Initialization interventions | 0..* | Temporary fixing, relaxed constraints, branch seeds and restoration/evidence obligations. | Original problem remains retained; reversible interventions have explicit scope. |

**Permitted variants:** Explicit algebraic property equations. Residual callback with analytical or qualified numerical derivatives. Opaque value evaluator in a supported host formulation.

**Invariants:** A value callback does not imply symbolic equations or derivatives. Temporary fixed variables must be restored or explicitly reconciled before acceptance of the original formulation. Structural phase candidates for an equation model do not certify all of them are physically present.

**Lifecycle and mutability:** Bound to captured model construction; changes to equations or coordinate bindings require reassessment before state restoration.

**Related concepts:** [IC-04](#ic-04), [IC-15](#ic-15), [IC-34](#ic-34), [IC-36](#ic-36), [IC-37](#ic-37), [IC-38](#ic-38), [IC-39](#ic-39), [IC-41](#ic-41).

**Valid interpretation:** A stage contribution supplies phase enthalpies and equilibrium relations while the column owns interstage balances.

**Invalid interpretation:** The property block and host independently prescribe conflicting phase fractions.

**Requirement basis:** [FR-GOV-04](thermodynamics_functional_requirements_v0_1.md#fr-gov-04), [FR-PRP-06](thermodynamics_functional_requirements_v0_1.md#fr-prp-06), [FR-FLW-07](thermodynamics_functional_requirements_v0_1.md#fr-flw-07), [FR-RUN-02](thermodynamics_functional_requirements_v0_1.md#fr-run-02). **Global rules:** SI-20. **Worked examples:** SE-06, SE-12.

<a id="ic-41"></a>
#### IC-41. Postcondition and check obligation

**Owner:** P05. **Kind:** definition. **Step-5 product:** A09.

The required evidence for a calculation to meet its claimed scope: targets, conservation, authority, phase/chemical conditions or derivative correctness.

**Identity:** Check identity fixes the original subject/request, mathematical criterion, tolerance/scale policy and mandatory/optional applicability.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Criterion and target | 1 | Original observable or relationship to test, including conservation and authority invariants. | Reference target captured before solving; not replaced by candidate. |
| Scope and applicability | 1 | Property, equilibrium, stage, unit or flowsheet scope; mandatory, conditional or optional role. | Condition under which the check becomes required. |
| Residual definition | 1 | How input/candidate/exchange values form the physical residual, independent of solver success. | Declared sign, dimensions and reference convention. |
| Tolerance and scale policy | 1 | Predeclared absolute/relative criteria, scale definition, floor and zero-scale behavior. | No universal accuracy percentage; gross scales for cancellation-prone balances. |
| Evidence needed | 1..* | Required candidate fields, evaluations, reference observations or process residuals. | Same-model reevaluation distinguished from independent reference evidence. |
| Unavailable-evidence outcome | 1 | Mandatory unknown blocks that scope; optional unknown remains explicitly unassessed. | Not-assessed is not pass. |

**Permitted variants:** PH target residual. Material/energy conservation including declared exchange. Independent validation against a bounded datum set.

**Invariants:** A local flash check cannot certify a whole recycle. A small step size does not replace a target-residual check. No check may silently adjust an original constraint or add a reservoir to pass.

**Lifecycle and mutability:** Defined with original problem or qualification plan; assessment results live in IC-59.

**Related concepts:** [IC-05](#ic-05), [IC-10](#ic-10), [IC-21](#ic-21), [IC-36](#ic-36), [IC-37](#ic-37), [IC-38](#ic-38).

**Valid interpretation:** A result reporting solver success with h=20 against target 50 fails the original enthalpy criterion.

**Invalid interpretation:** Changing the target to 20 after the solve and declaring zero residual.

**Requirement basis:** [FR-EQL-02](thermodynamics_functional_requirements_v0_1.md#fr-eql-02), [FR-EQL-03](thermodynamics_functional_requirements_v0_1.md#fr-eql-03), [FR-EQL-06](thermodynamics_functional_requirements_v0_1.md#fr-eql-06), [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02), [FR-CHM-02](thermodynamics_functional_requirements_v0_1.md#fr-chm-02), [FR-CHM-08](thermodynamics_functional_requirements_v0_1.md#fr-chm-08), [FR-RES-02](thermodynamics_functional_requirements_v0_1.md#fr-res-02). **Global rules:** SI-18, SI-24. **Worked examples:** SE-01, SE-14.

<a id="ic-42"></a>
#### IC-42. Effective physical restriction and branch request

**Owner:** P05. **Kind:** definition. **Step-5 product:** A09.

The actual local restrictions and branch expectations of a problem, distinct from reusable package defaults, numerical starting guesses, and phases an algorithm happened to inspect.

**Identity:** Identity fixes all effective restrictions, rationale and branch meaning under an exact base policy revision.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Base eligibility | 1 | Physical candidates/default policies being specialized. | Policy revision. |
| Restrictions and permissions | 0..* | Excluded phases, suppressed reactions, frozen constituents, allowed exchanges and other actual physical limits. | Scope and rationale for every specialization. |
| Requested branch | 0..1 | Stable, constrained, metastable or explicitly specified continuation branch; absence means use declared default policy, not arbitrary root. | A branch seed alone is not a physical restriction. |
| Structural/physical candidate distinction | 1 | What the formulation allocates versus what the physical problem admits. | Numerical examined set reported separately in execution evidence. |
| Claim limitations | 1 | What equilibrium/stability claims remain admissible under these restrictions. | Unrestricted stability not inferred from constrained success. |

**Permitted variants:** Inert transported solids excluded from equilibrium. Solid suppression with a constrained-fluid branch. Frozen chemical subsets.

**Invariants:** Changing restrictions changes the effective physical problem even if the reusable package is unchanged. An imposed phase can be unsupported or physically inadmissible and must not guarantee success.

**Lifecycle and mutability:** Captured per problem; lifting suppression creates a new problem and invalidates applicability of the restricted result to it.

**Related concepts:** [IC-22](#ic-22), [IC-45](#ic-45), [IC-46](#ic-46).

**Valid interpretation:** A suppressed-solid value is retained as restricted history when unrestricted precipitation is later enabled.

**Invalid interpretation:** A constrained liquid root is labeled a globally stable equilibrium without assessing allowed competitors.

**Requirement basis:** [FR-CFG-05](thermodynamics_functional_requirements_v0_1.md#fr-cfg-05), [FR-CFG-07](thermodynamics_functional_requirements_v0_1.md#fr-cfg-07), [FR-EQL-06](thermodynamics_functional_requirements_v0_1.md#fr-eql-06), [FR-EQL-07](thermodynamics_functional_requirements_v0_1.md#fr-eql-07), [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05), [FR-EXT-04](thermodynamics_functional_requirements_v0_1.md#fr-ext-04). **Global rules:** SI-09, SI-10. **Worked examples:** SE-15.

### P06 — Chemistry definitions and participation

<a id="ic-43"></a>
#### IC-43. Reaction identity and stoichiometry

**Owner:** P06. **Kind:** definition. **Step-5 product:** A05.

The chemical transformation and extent convention, independently of the kinetic law, equilibrium closure, reactor geometry and local achieved conversion.

**Identity:** Identity fixes species roles, normalized stoichiometry, phase conventions and extent normalization. Rescaling coefficients changes extent coordinates unless mapped explicitly.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Species and stoichiometric coefficients | 1..* | Reactant/product signs, identities and phase-specific species meanings when needed. | Moles of species per unit defined reaction extent or another explicit justified basis. |
| Extent/base-reactant convention | 1 | Normalization and relation between extent, specified conversion and selected base reactant. | Extent or rate units explicit; molecular total moles need not be conserved. |
| Conserved-basis qualification | 1 | Which balances stoichiometry satisfies and which remain unjustified. | For a linear species basis, A nu=0 for each claimed closed conserved row. |
| Reference/property requirements | 0..* | Formation/standard-state data and energy correction requirements exported for package compatibility. | No independent unconditional addition of reaction heat. |

**Permitted variants:** Conversion reaction. Reaction set component used by equilibrium or rate laws. Solid/gas reaction with identified forms.

**Invariants:** Unbalanced asserted chemistry is rejected or unresolved for that claim, not repaired by normalization. Strict conversion feasibility is checked before adoption; clipping negative amounts is not the same request.

**Lifecycle and mutability:** Revision when chemistry or extent convention changes; residence time changes do not change this identity.

**Related concepts:** [IC-01](#ic-01), [IC-05](#ic-05), [IC-12](#ic-12).

**Valid interpretation:** For A→B and one mole A available, extent 0.4 leaves 0.6 A,0.4 B; strict extent 1.2 is infeasible.

**Invalid interpretation:** Treating a limited extent of one as successful fulfillment of a strict requested extent of 1.2.

**Requirement basis:** [FR-MAT-08](thermodynamics_functional_requirements_v0_1.md#fr-mat-08), [FR-CHM-01](thermodynamics_functional_requirements_v0_1.md#fr-chm-01), [FR-CHM-02](thermodynamics_functional_requirements_v0_1.md#fr-chm-02), [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03), [FR-CHM-05](thermodynamics_functional_requirements_v0_1.md#fr-chm-05). **Global rules:** SI-15. **Worked examples:** SE-03, SE-04.

<a id="ic-44"></a>
#### IC-44. Reaction-law and thermochemical requirement

**Owner:** P06. **Kind:** definition. **Step-5 product:** A05.

The relationship used for a declared reaction: specified conversion, equilibrium relation or finite-rate law, with its independent variables and normalization.

**Identity:** Law identity fixes reaction revision, mathematical meaning, input standard states/bases and parameter interpretations.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Reaction target | 1..* | Reaction(s) or coherent chemical relation to which the law applies. | Stoichiometry and extent normalization. |
| Law kind and relationship | 1 | Specified extent/conversion constraint, equilibrium constant relation or kinetic expression. | Dimensionless equilibrium convention or explicit dimensional law basis. |
| Input/property requirements | 1..* | Activities, fugacities, concentrations, temperature and other requested quantities as exported semantic meanings. | Concentration frame, solvent basis, standard pressure, activity definition as applicable. |
| Parameters and applicability | 0..* | Rate/equilibrium coefficients with exact equation units and source range. | Ordered coefficients and thermochemical interpretation. |
| Rate normalization | 0..1 | Required for kinetics: per volume, catalyst mass, area or other denominator and relation to process totals. | Local rate versus total extent rate distinguished. |

**Permitted variants:** Fixed-conversion rule. Standard-state-qualified equilibrium relation. Kinetic law with explicit catalyst/site normalization.

**Invariants:** A reaction rate per kg catalyst is not automatically per reactor volume. Reaction order need not be inferred from stoichiometric coefficient. A finite-rate law cannot be replaced by equilibrium because only equilibrium is available.

**Lifecycle and mutability:** Used as reusable definition; local rates/extent values are quantity assertions in a captured problem/result.

**Related concepts:** [IC-09](#ic-09), [IC-12](#ic-12), [IC-14](#ic-14), [IC-15](#ic-15), [IC-43](#ic-43).

**Valid interpretation:** The same kinetic law can be evaluated at several reactor locations without owning reactor residence time.

**Invalid interpretation:** A rate formula expecting molarity receives mole fractions with no conversion.

**Requirement basis:** [FR-CHM-01](thermodynamics_functional_requirements_v0_1.md#fr-chm-01), [FR-CHM-02](thermodynamics_functional_requirements_v0_1.md#fr-chm-02), [FR-CHM-04](thermodynamics_functional_requirements_v0_1.md#fr-chm-04), [FR-CHM-05](thermodynamics_functional_requirements_v0_1.md#fr-chm-05).

<a id="ic-45"></a>
#### IC-45. Chemical system and participation description

**Owner:** P06. **Kind:** definition. **Step-5 product:** A05.

The coherent species/reaction domain and participation assumptions needed for property, reactive equilibrium, or kinetic/equilibrium calculations.

**Identity:** Chemical-system revision fixes species meanings, data, admissible reactions/domains, conserved basis and participation rules. Local specialized restrictions remain captured in the problem.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Species and representations | 1..* | Permitted true species and any apparent representation relationships. | Exact identities; zero-feed products remain possible if eligible. |
| Reaction/law and data set | 0..* | Explicit reactions/laws or a database-based chemical-potential system without an enumerated reaction list. | Thermochemical data/conventions kept coherent. |
| Domain/participation rules | 1..* | Equilibrated, kinetic, frozen, inert or excluded species/reactions in their domains. | Frozen species, frozen phase allocation and zero initial amount are different. |
| Conserved quantities | 1..* | Totals constrained in closed or explicitly open chemical problems. | Element/mass/charge/sites only where justified. |
| Exchange requirements | 0..* | Allowed-to-be-requested reservoirs/titrants and their implications; unit must authorize actual exchange. | Open-system definitions. |

**Permitted variants:** Reaction-equation system. Chemical-potential database system. Mixed kinetic/equilibrated/frozen subsets.

**Invariants:** One coupled problem may resolve chemistry and phases simultaneously. A fixed-species VLE flash is not chemical equilibration. Choosing fewer species is an explicit model restriction, not just a faster equivalent solve.

**Lifecycle and mutability:** P06 owns chemistry meaning; P03 reconciles selected calorics; actual problems and solves reference a fixed revision.

**Related concepts:** [IC-01](#ic-01), [IC-03](#ic-03), [IC-05](#ic-05), [IC-06](#ic-06), [IC-14](#ic-14), [IC-43](#ic-43), [IC-44](#ic-44), [IC-46](#ic-46), [IC-47](#ic-47).

**Valid interpretation:** A kinetic mineral can coexist with an equilibrated aqueous subset without forcing mineral equilibrium.

**Invalid interpretation:** A solver adds a new possible product species in a retry without changing the problem identity.

**Requirement basis:** [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03), [FR-CHM-04](thermodynamics_functional_requirements_v0_1.md#fr-chm-04), [FR-CHM-08](thermodynamics_functional_requirements_v0_1.md#fr-chm-08), [FR-LIF-02](thermodynamics_functional_requirements_v0_1.md#fr-lif-02). **Global rules:** SI-15. **Worked examples:** SE-03.

<a id="ic-46"></a>
#### IC-46. Reservoir and chemical-exchange requirement

**Owner:** P06. **Kind:** definition. **Step-5 product:** A05.

The material/energy exchange semantics required by a chemical constraint, not a silent license for the numerical provider to introduce matter.

**Identity:** Requirement identity fixes reservoir/substance, imposed condition, allowed exchange coordinates and reference convention.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Reservoir/substance identity | 1 | External gas, titrant, component source/sink, electron or other explicitly modeled reservoir meaning. | Composition/charge/energy conventions where justified. |
| Imposed chemical condition | 1 | Target pH/fugacity/other constrained chemical condition and its convention. | Condition meaning and standard state are explicit. |
| Exchange dimensions and bounds | 1..* | Permitted amount/rate, direction and bounds; closed boundary is a distinct prohibition. | Sign convention relative to the material account. |
| Caloric/accounting requirements | 1 | Data required to include the exchange in a process balance; unknown energy blocks an energy-balanced claim where necessary. | No heat double counting or unreported substance addition. |
| Process authorization needed | 1 | Which local process choice must permit the reservoir and capacity before solving. | Requirement does not grant permission by itself. |

**Permitted variants:** Explicit titrant addition. Gas reservoir exchange under fugacity constraint. No exchange permitted for a closed system.

**Invariants:** An imposed chemical condition can be infeasible under a closed boundary; hidden exchange is prohibited. Every used exchange must appear as IC-71 in the candidate account and balance evidence.

**Lifecycle and mutability:** Definition is reusable; permitted actual exchange and its realization belong to a captured local problem and result.

**Related concepts:** [IC-01](#ic-01), [IC-03](#ic-03), [IC-05](#ic-05), [IC-12](#ic-12).

**Valid interpretation:** A pH-controlled case reports the titrant quantity and its relevant energy contribution.

**Invalid interpretation:** A closed-system solver satisfies a gas condition by unreported gas addition.

**Requirement basis:** [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03), [FR-CHM-07](thermodynamics_functional_requirements_v0_1.md#fr-chm-07). **Global rules:** SI-16. **Worked examples:** SE-09.

<a id="ic-47"></a>
#### IC-47. Apparent/true-species reconciliation

**Owner:** P06. **Kind:** definition. **Step-5 product:** A05.

The chemistry-aware relation between feed/reporting components and true species of the same material. It may require solving and may have a nonunique reverse attribution.

**Identity:** Reconciliation revision captures both representations, conserved mappings, chemical conventions and any reporting attribution rules.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Source/target descriptions | 1 | Apparent and true representations and applicable coordinate maps. | Exact revisions and amount bases. |
| Shared conserved account | 1 | Relations that ensure both descriptions denote the same underlying material totals. | Element/mass/charge conventions justified for this system. |
| Chemical constraints | 0..* | Speciation or reaction conditions required to determine the true species; empty only for a justified direct map. | Chemical system and participation requirements. |
| Reverse reporting rule | 1 | Unique mapping, constrained attribution, convention-dependent reporting or unavailable inverse. | Any assumed association is reported as a convention, not observed molecules. |
| No-duplicate-inventory rule | 1 | Which actual material account is represented and how aliases/views are excluded from additive balances. | One physical account, multiple descriptive views. |

**Permitted variants:** Feed formula-to-element totals followed by speciation. Reporting salt-equivalent convention. Direct known species relabeling where genuinely equivalent.

**Invariants:** Apparent salt plus its represented ions must not be summed as distinct inventory. Preserving element totals does not generally choose a unique apparent-species attribution. State-dependent speciation is not mislabeled a constant permutation matrix.

**Lifecycle and mutability:** Revise when chemical/reporting interpretation changes; applications retain exact state and assumption lineage.

**Related concepts:** [IC-03](#ic-03), [IC-05](#ic-05), [IC-07](#ic-07), [IC-45](#ic-45).

**Valid interpretation:** One mol apparent NaCl represented as one mol Na+ and one mol Cl− preserves elements/charge accounting but not molecular mole count.

**Invalid interpretation:** The report counts one mol NaCl plus both ions as additional physical matter.

**Requirement basis:** [FR-CHM-06](thermodynamics_functional_requirements_v0_1.md#fr-chm-06), [FR-FLW-06](thermodynamics_functional_requirements_v0_1.md#fr-flw-06). **Global rules:** SI-06, SI-16. **Worked examples:** SE-03.

### P07 — Flowsheet integration and use-case coordination

<a id="ic-48"></a>
#### IC-48. Process location and region description

**Owner:** P07. **Kind:** definition. **Step-5 product:** A07.

The named place in the host process model where thermodynamic material information is used: stream endpoint, stage, segment, bulk/interface, inventory or other domain.

**Identity:** Host location identity plus structural revision and role. A graphical label or screen position is not physical identity.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Host object and local role | 1 | Unit/connection reference plus stage, segment, side, interface or inventory designation. | Opaque host identity; geometry remains host-owned. |
| Location hierarchy | 0..1 | Parent region/location and declared inheritance context; root has no parent. | Hierarchy used for binding resolution, not solver order. |
| Material-domain meaning | 1..* | Applicable representation/domain description and quantity purpose. | No implicit flow for inventories or interfaces. |
| Time association | 0..1 | Physical time/slice if relevant; absent for a steady state; distinct from execution timestamps. | Time origin and units explicit where values are supplied. |
| Interfaces/couplings | 0..* | Other locations to which heat, material or constitutive relationships connect. | Coupling kind explicit. |

**Permitted variants:** Column stage or spatial cell. Separate bulk and interface locations. Closed inventory location.

**Invariants:** Two exchanger sides remain different material locations. Internal states do not require graphical stream objects. A location parent hierarchy cannot contain an unexplained inheritance cycle.

**Lifecycle and mutability:** Host owns topology; changes to relevant structure trigger new bindings/problems and dependency reassessment.

**Related concepts:** [IC-03](#ic-03), [IC-06](#ic-06), [IC-48](#ic-48), [IC-50](#ic-50).

**Valid interpretation:** COLUMN-1/stage-3/vapor is a stable local role independent of icon position.

**Invalid interpretation:** Moving an icon changes its thermodynamic region or material identity.

**Requirement basis:** [FR-CFG-06](thermodynamics_functional_requirements_v0_1.md#fr-cfg-06), [FR-STA-01](thermodynamics_functional_requirements_v0_1.md#fr-sta-01), [FR-FLW-01](thermodynamics_functional_requirements_v0_1.md#fr-flw-01), [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08).

<a id="ic-49"></a>
#### IC-49. Effective thermodynamic location binding

**Owner:** P07. **Kind:** definition. **Step-5 product:** A07.

The resolved thermodynamic assignment for a location after explicit defaults, inheritance and permitted local specialization have been applied.

**Identity:** Binding revision fixes location, base package, local physical overlays and resolution trace; no dependence on order of evaluations.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Location | 1 | The process location/region to which the binding applies. | Exact structural context. |
| Base resolved package | 1 | Reusable coherent package applied at this location. | Package revision. |
| Local overlays | 0..* | Permitted phase/chemistry/branch specialization and reporting references. | Physical effect and source of override explicitly recorded. |
| Resolution trace | 1 | Explicit assignment versus inherited/default choices, precedence and conflict resolution. | No last-used-session rule. |
| Boundary compatibility needs | 0..* | Neighboring material bindings requiring translation or separate heat-only treatment. | Identity/reference/operation prerequisites. |

**Permitted variants:** Direct explicit binding. Inherited binding with no override. Allowed local restricted-chemistry specialization.

**Invariants:** Ambiguous inheritance blocks affected calculation rather than choosing the latest used package. Changing a local restriction changes effective problem meaning without necessarily editing the shared package.

**Lifecycle and mutability:** P07 resolves proposals; P10 commits binding revision and invalidates affected contexts.

**Related concepts:** [IC-20](#ic-20), [IC-42](#ic-42), [IC-48](#ic-48), [IC-51](#ic-51).

**Valid interpretation:** Two exchanger sides use different packages throughout interleaved calls.

**Invalid interpretation:** A stage accidentally uses the package left active by the previous stage.

**Requirement basis:** [FR-CFG-06](thermodynamics_functional_requirements_v0_1.md#fr-cfg-06), [FR-FLW-05](thermodynamics_functional_requirements_v0_1.md#fr-flw-05). **Worked examples:** SE-05.

<a id="ic-50"></a>
#### IC-50. Unit demands, coupling and routing declaration

**Owner:** P07. **Kind:** definition. **Step-5 product:** A18.

The thermodynamic boundary of an equipment/process formulation: required properties, balance constraints, material/heat transfer and product allocation rules.

**Identity:** Declaration revision fixes unit mode, relevant locations, assumptions and completion hierarchy; it does not own global solver implementation.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Unit mode and locations | 1 | Selected physical formulation and participating input/internal/output locations. | Unit-owned geometry referenced only as needed for input semantics. |
| Property/contribution demands | 1..* | Mandatory and optional local properties, derivatives or equations needed by that mode. | Demand scope and conditionality retained. |
| Balance and operating constraints | 1..* | Pressure/temperature/heat/work/efficiency and local coupling relations supplied by unit. | Signs, bases, required physical assumptions explicit. |
| Product-routing and transfer rules | 0..* | Mechanical split, phase routing, entrainment/carryover, or interfacial transfer convention. | Port roles differ from phase identities; geometry-dependent closures remain unit-owned. |
| Completion and check scope | 1 | Required local/unit/flowsheet conditions and which candidate set must be consistent. | No local-success implication for parent scope. |

**Permitted variants:** Mixer/mechanical splitter. Separator with explicit solids carryover. Two-sided exchanger. Rate-based contactor with distinct bulk/interface states.

**Invariants:** A duty-only heater is a different demand from its rating mode. A separator routes evaluated phases under named assumptions rather than making ports physical phase identities. A heat-only connection does not perform constituent translation.

**Lifecycle and mutability:** Changes in formulation or physical rules produce new declarations and affected problems; display-only edits do not.

**Related concepts:** [IC-34](#ic-34), [IC-36](#ic-36), [IC-38](#ic-38), [IC-40](#ic-40), [IC-41](#ic-41), [IC-48](#ic-48), [IC-52](#ic-52).

**Valid interpretation:** Transport data become mandatory for exchanger rating while enthalpy-only duty remains separately supported.

**Invalid interpretation:** Ignoring missing viscosity and presenting duty-only results as completed rating.

**Requirement basis:** [FR-EQL-03](thermodynamics_functional_requirements_v0_1.md#fr-eql-03), [FR-FLW-01](thermodynamics_functional_requirements_v0_1.md#fr-flw-01), [FR-FLW-02](thermodynamics_functional_requirements_v0_1.md#fr-flw-02), [FR-FLW-03](thermodynamics_functional_requirements_v0_1.md#fr-flw-03), [FR-FLW-04](thermodynamics_functional_requirements_v0_1.md#fr-flw-04), [FR-FLW-07](thermodynamics_functional_requirements_v0_1.md#fr-flw-07), [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08). **Worked examples:** SE-02, SE-07, SE-08.

<a id="ic-51"></a>
#### IC-51. Material-boundary translation case

**Owner:** P07. **Kind:** problem. **Step-5 product:** A18.

An application of representation and convention mappings between particular source and target process descriptions under explicitly preserved constraints.

**Identity:** Translation-case identity captures source/target binding/snapshot, map revisions, chosen preserved quantities and authorized physical exchanges.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Source and target binding | 1 | Both sides of the material boundary. | Exact effective package/representation context. |
| Source material and representation map | 1 | Actual source snapshot and applicable map, including any chemical/reporting reconciliation. | Conserved bases and information-loss statement. |
| Reference transformation | 0..1 | Known energy/standard-state relation required for the requested preservation; absent relation means unassessed/incomplete where needed. | May depend on composition/domain; no guessed universal offset. |
| Preserved quantities and target problem | 1 | T,P,composition or P,corrected-H,etc. as a jointly admissible set. | Do not preserve incompatible values by hidden correction. |
| Discrepancy and actual exchanges | 1 | Model disagreement and mapping residual recorded separately from genuinely physical heat/work/material transfers. | Each term has declared dimensions and sign. |

**Permitted variants:** Same species with two caloric packages. Detailed-to-lumped representation. Apparent-to-true reactive-system boundary.

**Invariants:** Mapping, reference conversion and target-state resolution are distinct steps in meaning, even if one implementation combines them. A nonreversible map requires additional assumptions to reconstruct details. Heat-only coupling is IC-50 and does not instantiate a material translator.

**Lifecycle and mutability:** Case fixed per attempted translation; changed policies or sources create another case and remain attributable.

**Related concepts:** [IC-07](#ic-07), [IC-21](#ic-21), [IC-28](#ic-28), [IC-35](#ic-35), [IC-36](#ic-36), [IC-47](#ic-47), [IC-49](#ic-49), [IC-71](#ic-71).

**Valid interpretation:** Holding T,P,composition exposes corrected enthalpy disagreement; holding corrected H may require another T.

**Invalid interpretation:** Inserting an unexplained heat term to preserve incompatible T and H predictions simultaneously.

**Requirement basis:** [FR-FLW-05](thermodynamics_functional_requirements_v0_1.md#fr-flw-05), [FR-FLW-06](thermodynamics_functional_requirements_v0_1.md#fr-flw-06). **Global rules:** SI-14. **Worked examples:** SE-04, SE-05.

<a id="ic-52"></a>
#### IC-52. Process completion scope and run-local view

**Owner:** P07. **Kind:** description. **Step-5 product:** A18.

The scope at which a coupled calculation is progressing or must be accepted, together with the candidate set used for further iterations.

**Identity:** Scope identity fixes operation/stage/unit/flowsheet role and parent; run-local view identity fixes captured context and iteration. It is not a published current result.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Scope and parent | 1 | Local property/equilibrium, stage, unit or flowsheet scope; a parent relation is optional only for roots. | Containment does not imply parent convergence. |
| Captured revision/run references | 1 | Exact process/dependency context and permitted run supplied by P10. | Reference values; source of cancellation authority remains P10. |
| Candidate collection | 0..* | The local candidates/qualified states used in this iteration, with compatibility context. | No mixture of incompatible parameter revisions or material coordinates. |
| Process residuals and criteria | 1..* | Unit/flowsheet constraints, recycle comparisons and required completion evidence. | Predeclared residual and tolerance definitions. |
| Publication-group intent | 1 | Which related locations/results are to be adopted together at the claimed scope. | Diagnostic partial results are separately labeled. |

**Permitted variants:** Sequential recycle iteration view. Coupled equation solve candidate set. A unit-local checking scope within a larger run.

**Invariants:** Local results may feed the next iteration without being accepted whole-flowsheet results. Exhausted parent iteration remains unaccepted even if every child flash converged. Related outlet states cannot be published as a completed unit set from incompatible runs.

**Lifecycle and mutability:** New view per iteration or coherent update; retain history separately from P10's current binding.

**Related concepts:** [IC-41](#ic-41), [IC-52](#ic-52), [IC-56](#ic-56), [IC-60](#ic-60).

**Valid interpretation:** A converged flash inside an unconverged recycle is inspectable and usable within that run only at its achieved scope.

**Invalid interpretation:** Two locally good exchanger outputs from different parameter revisions are published as one accepted exchanger state.

**Requirement basis:** [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08), [FR-RES-04](thermodynamics_functional_requirements_v0_1.md#fr-res-04). **Global rules:** SI-23. **Worked examples:** SE-13.

### P08 — Provider realization and numerical execution

<a id="ic-53"></a>
#### IC-53. Provider realization and capability attestation

**Owner:** P08. **Kind:** definition. **Step-5 product:** A11.

The actual engine/build/binding/data realization and the operation combinations that the adapter exposes. It does not inherit every capability of an upstream project.

**Identity:** Realization identity fixes engine artifact, build/options, binding/adapter revision, linked data and relevant environment. Attestations are separately versioned evidence about that realization.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Implementation/dependency manifest | 1 | Engine, adapter, binding, enabled modules, exact data bundles and known availability conditions. | Exact artifact identities; required missing artifacts remain explicit. |
| Semantic realization mapping | 1..* | Which configured methods and operation contracts are implemented, including whole-bundle versus submodel boundary. | Mapped role/convention meanings, not names alone. |
| Capability predicates | 1..* | Supported, unsupported or unassessed combinations of representation/model, phases, input specifications, properties, derivatives and initializer conditions. | Conditions and evidence levels; method existence is not runtime validation. |
| Provider coordinates | 0..* | Native order, units and phase/species mappings for exact realizations. | No provider coordinates become canonical material identity. |
| Lifecycle constraints | 1 | Sharing, reentrancy, activation, failure/cancellation and reference-state lifetime qualifications. | Unknown lifecycle behavior requires conservative control, not a false guarantee. |
| Opacity and limitations | 1 | Which internal choices are attributable and which remain unknown; claim limits and unresolved dependencies. | Opaque but pinned data permitted with bounded reproduction claims. |

**Permitted variants:** Whole configured engine. Coherent chemical-system engine. Selected kernels or equation contribution.

**Invariants:** A wrapper available without its required engine is not executable. Source inspection, exercised execution and independent validation are separate levels. An adapter may realize several conceptual packages internally without replacing host authority.

**Lifecycle and mutability:** Revision on relevant implementation/data changes; sessions and results retain exact realization identity.

**Related concepts:** [IC-10](#ic-10), [IC-20](#ic-20), [IC-24](#ic-24), [IC-34](#ic-34), [IC-55](#ic-55), [IC-58](#ic-58).

**Valid interpretation:** A TP-only adapter declares PH unsupported although the upstream library has a PH implementation elsewhere.

**Invalid interpretation:** A solids parameter whose implementation rejects solids is registered as working precipitation.

**Requirement basis:** [FR-GOV-01](thermodynamics_functional_requirements_v0_1.md#fr-gov-01), [FR-GOV-02](thermodynamics_functional_requirements_v0_1.md#fr-gov-02), [FR-GOV-03](thermodynamics_functional_requirements_v0_1.md#fr-gov-03), [FR-GOV-04](thermodynamics_functional_requirements_v0_1.md#fr-gov-04), [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03), [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08), [FR-RUN-01](thermodynamics_functional_requirements_v0_1.md#fr-run-01), [FR-RUN-03](thermodynamics_functional_requirements_v0_1.md#fr-run-03). **Global rules:** SI-22, SI-28. **Worked examples:** SE-14.

<a id="ic-54"></a>
#### IC-54. Execution plan and attempt record

**Owner:** P08. **Kind:** execution. **Step-5 product:** A12.

How one captured physical problem will be attempted, or was attempted, using a compatible realization and numerical policy. An attempt is not the physical problem itself.

**Identity:** Attempt identity is unique within a run and references exactly one original or explicitly alternative physical problem. Retry parentage retained.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Problem and captured context | 1 | Original physical problem plus supplied revision/run token and intended completion scope. | No mutable reference to current inputs. |
| Realization and numerical policy | 1 | Chosen eligible provider and configured algorithms/limits. | Independent of physical-policy identity. |
| Initialization/continuation input | 0..* | Guesses, phase seeds, prior compatible result references and eligibility evidence. | Numerical role and validity bound explicit. |
| Attempt relations and interventions | 0..* | Retry parent, altered-problem proposal, temporary fixes or relaxations and restoration checks. | Changed physics points to another IC-35; no hidden rewrite. |
| Execution state and evidence | 1 | Planned, running, terminated, cancelled or failed and actual used methods/search/termination details. | Execution times differ from physical process time. |

**Permitted variants:** One direct state solve. Nested inverse solve or continuation. One parameter-fit evaluation under a fixed trial package.

**Invariants:** A plan can be rejected before provider evaluation. Numerical relaxation or temporary fixing is not accepted as the original problem until original checks are restored. A new attempt must not reclassify an unapproved alternate model as same-problem success.

**Lifecycle and mutability:** Plan/capture then execute; completed records are evidence. Work buffers live in session/workspace and do not become source definitions.

**Related concepts:** [IC-35](#ic-35), [IC-41](#ic-41), [IC-53](#ic-53), [IC-54](#ic-54), [IC-56](#ic-56), [IC-57](#ic-57), [IC-63](#ic-63), [IC-70](#ic-70), [IC-72](#ic-72).

**Valid interpretation:** Two different algorithms can attempt the same PH target with distinct attempt records.

**Invalid interpretation:** The second attempt narrows physical phase scope but keeps the same original problem identity unqualified.

**Requirement basis:** [FR-DAT-04](thermodynamics_functional_requirements_v0_1.md#fr-dat-04), [FR-RUN-02](thermodynamics_functional_requirements_v0_1.md#fr-run-02), [FR-RUN-04](thermodynamics_functional_requirements_v0_1.md#fr-run-04), [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05), [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06), [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08), [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07). **Global rules:** SI-21. **Worked examples:** SE-01, SE-11, SE-12, SE-13.

<a id="ic-55"></a>
#### IC-55. Session and workspace lifecycle record

**Owner:** P08. **Kind:** execution. **Step-5 product:** A12.

The declared compatibility and actual health of a provider runtime context, including its private numerical buffers. A handle is not a material definition or saved scientific result.

**Identity:** Session identity is runtime-scoped; compatibility key references exact relevant configuration/realization/conventions. Recreated sessions get new runtime identity even when semantically equivalent.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Realization and compatibility key | 1 | Provider/build/binding and exact material/data/convention/coordinate revisions required for safe reuse. | No mutable current-stream pointer as scientific identity. |
| Sharing/isolation contract | 1 | Exclusive, serialized sequence, independently reentrant or other tested/qualified sharing policy. | Internal parallelism does not imply host-level independent session safety. |
| Workspace and active-call association | 0..* | Private guesses, scratch arrays and active attempt ownership with explicit isolation scope. | Runtime state only; no persisted authority from buffers. |
| Health and recovery evidence | 1 | Usable, uncertain, quarantined, reconstructed, retired or unassessed health and conditions for reuse. | A previous successful result does not validate a failed retained buffer. |
| Reference/configuration invalidation | 1 | Events that require rebuilding, resetting or proving compatibility before reuse. | Changed reference, material coordinates or parameter bundle explicit. |

**Permitted variants:** Reusable isolated backend state. Serialized activation-plus-evaluation session. One-shot provider process or disposable context.

**Invariants:** No canonical material/state/acceptance authority is transferred to native working memory. If safe recovery is unknown, retire/reconstruct rather than assume unchanged state. Opaque native handles cannot be serialized as reconstructable physics.

**Lifecycle and mutability:** Create -> qualified use -> reuse assessment or quarantine -> reset/rebuild/retire. No execution guarantee is inferred without build-specific testing.

**Related concepts:** [IC-20](#ic-20), [IC-53](#ic-53), [IC-54](#ic-54), [IC-58](#ic-58).

**Valid interpretation:** A failed native update retires the session while the host retains the previously accepted state.

**Invalid interpretation:** Another request reads leftover properties from a failed work buffer as if newly calculated.

**Requirement basis:** [FR-RUN-03](thermodynamics_functional_requirements_v0_1.md#fr-run-03), [FR-RUN-04](thermodynamics_functional_requirements_v0_1.md#fr-run-04), [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06), [FR-RUN-07](thermodynamics_functional_requirements_v0_1.md#fr-run-07). **Global rules:** SI-22. **Worked examples:** SE-14.

<a id="ic-56"></a>
#### IC-56. Provider candidate and execution evidence

**Owner:** P08. **Kind:** evidence. **Step-5 product:** A13.

The normalized values or mathematical contribution actually returned, plus how they were obtained and what failed or remained uncomputed. It is not an acceptance verdict.

**Identity:** Candidate identity fixes producing attempt/problem and payload snapshot; later evidence augments lineage without changing original values.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Attempt and original problem | 1 | Exact source attempt and physical question. | Captured revisions and conventions. |
| Returned payload | 0..* | Normalized states, quantity assertions, derivatives or contribution data; may be partial or absent on failure. | Native ordering/units converted with explicit maps. |
| Per-output completion | 1..* | Computed, unavailable, not requested, failed or unassessed status for promised/required outputs. | Known zero separate from missing; role and scope explicit. |
| Actually used assumptions | 1 | Methods/data, phases searched, chemistry restrictions, branches and any departures from request. | Requested-versus-used differences retained. |
| Numerical evidence | 0..* | Termination criteria, residuals, iterations, conditioning and derivative method evidence where available. | Provider residuals not automatically independent physical checks. |
| Exchange realizations | 0..* | Actual modeled reservoir/material/energy exchanges needed by the solve. | Signed amounts/rates, basis and reference conventions. |

**Permitted variants:** Complete candidate state. Partial properties with optional failures. Failed solve with inspectable trial data.

**Invariants:** Candidate production does not authorize current publication. A numerical success with failed physical target remains a candidate with failed acceptance. Absent promised information is represented, not filled with default values.

**Lifecycle and mutability:** Immutable normalized evidence sent to P09; provider-private scratch arrays are detached or safely retained without aliasing accepted state.

**Related concepts:** [IC-26](#ic-26), [IC-28](#ic-28), [IC-35](#ic-35), [IC-40](#ic-40), [IC-42](#ic-42), [IC-54](#ic-54), [IC-63](#ic-63), [IC-71](#ic-71), [IC-72](#ic-72).

**Valid interpretation:** A flash returns phase allocation plus missing viscosity, matching a primary equilibrium/optional-viscosity contract.

**Invalid interpretation:** A candidate is considered fully populated because a library returned an object.

**Requirement basis:** [FR-EQL-05](thermodynamics_functional_requirements_v0_1.md#fr-eql-05), [FR-EQL-06](thermodynamics_functional_requirements_v0_1.md#fr-eql-06), [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02), [FR-PRP-06](thermodynamics_functional_requirements_v0_1.md#fr-prp-06), [FR-RUN-07](thermodynamics_functional_requirements_v0_1.md#fr-run-07), [FR-RES-01](thermodynamics_functional_requirements_v0_1.md#fr-res-01). **Global rules:** SI-10, SI-18. **Worked examples:** SE-01, SE-06, SE-14, SE-15.

<a id="ic-57"></a>
#### IC-57. Initialization and continuation description

**Owner:** P08. **Kind:** description. **Step-5 product:** A12.

Admissible starting information for an operation, including limitations and reversible numerical interventions. It cannot silently become an original user specification.

**Identity:** Identity fixes source context, guess/continuation values, preparation policy and compatibility assessment with target problem.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Target problem and preparation needs | 1 | Exact operation and initializer restrictions to be satisfied. | Phase region, chart, amounts and reference requirements explicit. |
| Seed values and origin | 0..* | User guesses, previous results, phase-envelope information, phase candidates or equation fixings. | Roles remain numerical seeds, not fixed physical inputs. |
| Compatibility/eligibility | 1 | Admissible, missing, failed or unassessed preparation and reason. | Failure of preparation is not proof of infeasibility. |
| Temporary interventions | 0..* | Fix/unfix, constraint relaxation or altered trial system and required restoration evidence. | Original problem retained and final checks refer to it. |
| Reuse limits | 1 | Conditions under which seeds remain meaningful after data, phase, coordinate or branch change. | No blindly shared stale guess with altered meanings. |

**Permitted variants:** Compatible prior state warm start. Two-phase initial-state prerequisite. Staged initialization for a coupled equation model.

**Invariants:** A missing initializer can block an otherwise representable/evaluable problem. An incipient phase seed is not present material. Initialization that changes physics is an explicitly temporary or alternative problem, not hidden production behavior.

**Lifecycle and mutability:** Prepare for one captured target; new target compatibility needs assessment; successful preparation not identical to solved state.

**Related concepts:** [IC-24](#ic-24), [IC-26](#ic-26), [IC-30](#ic-30), [IC-35](#ic-35), [IC-41](#ic-41).

**Valid interpretation:** A prior liquid composition seeds a new calculation while the original overall component totals remain fixed.

**Invalid interpretation:** A guessed temperature is left as an extra fixed constraint after initialization.

**Requirement basis:** [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03), [FR-STA-04](thermodynamics_functional_requirements_v0_1.md#fr-sta-04), [FR-EQL-07](thermodynamics_functional_requirements_v0_1.md#fr-eql-07), [FR-RUN-01](thermodynamics_functional_requirements_v0_1.md#fr-run-01), [FR-RUN-02](thermodynamics_functional_requirements_v0_1.md#fr-run-02). **Global rules:** SI-21. **Worked examples:** SE-12.

<a id="ic-58"></a>
#### IC-58. Provider coordinate and convention binding

**Owner:** P08. **Kind:** definition. **Step-5 product:** A11.

The executable mapping from semantic material/property coordinates into a particular provider and back. It instantiates P01 map meaning without changing canonical identity.

**Identity:** Binding revision fixes provider realization, host representation/coordinate revisions, native order, units, phase labels and conventions.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Host coordinate meaning | 1 | Semantic representation, composition charts and relevant quantity conventions. | Exact revisions. |
| Native coordinate description | 1 | Provider constituent order/IDs, supported bases, phase slots and output axes. | Native units/references are explicitly distinguished from host display units. |
| Bidirectional mappings | 1..* | Permutation, basis/reference conversions and result normalization, with direction-specific limits. | Incomplete/lossy transformations may have no valid inverse. |
| Native placeholders | 0..* | Numerical-only entries/sentinel meanings and rules excluding them from physical accounting. | No fabricated material, properties or capability from placeholder data. |
| Capability/context constraints | 1 | When mapping is admissible and which native lifecycle associations are required. | Bound to provider realization and exact material scope. |

**Permitted variants:** Simple constituent permutation. Mass/molar conversion with required data. Native fixed-slot phase result normalized into role-qualified entries.

**Invariants:** All vectors and derivative axes must use the matching coordinate map, not just composition. Native liquid1/liquid2 labels are not automatically physical phase history or light/heavy ports. Adapter-only placeholders cannot justify a physically unsupported operation.

**Lifecycle and mutability:** Rebuild/reassess when native/host order or definitions change; save mapping semantics, not a raw handle.

**Related concepts:** [IC-03](#ic-03), [IC-04](#ic-04), [IC-07](#ic-07), [IC-21](#ic-21), [IC-53](#ic-53).

**Valid interpretation:** Reversing component order remaps composition, property vectors and Jacobian axes consistently.

**Invalid interpretation:** Composition is remapped but binary parameters or derivatives retain the old order.

**Requirement basis:** [FR-GOV-02](thermodynamics_functional_requirements_v0_1.md#fr-gov-02), [FR-MAT-07](thermodynamics_functional_requirements_v0_1.md#fr-mat-07), [FR-RUN-04](thermodynamics_functional_requirements_v0_1.md#fr-run-04). **Global rules:** SI-01, SI-22.

<a id="ic-72"></a>
#### IC-72. Numerical policy

**Owner:** P08. **Kind:** definition. **Step-5 product:** A12.

The selected numerical procedure and its limits, not the physical problem or scientific validity envelope.

**Identity:** Policy revision fixes algorithms, termination/precision controls, search bounds, initialization and retry choices. Actual use remains recorded per attempt.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Algorithms and termination | 1..* | Eligible solution methods and numerical stopping/budget criteria. | Units/scales for tolerances; small step is not target acceptance. |
| Numerical bounds and scaling | 0..* | Search intervals, variable transforms and scaling used for numerics. | Distinct from physical admissibility/data validity. |
| Initializer and continuation policy | 1 | Required seeds/preparation and allowable reuse of compatible starts. | Does not promote guesses to fixed physical input. |
| Retry and alternate policy | 1 | Same-problem retries and separately authorized requests for changed-physics alternatives. | A policy cannot erase original-problem failure. |
| Derivative realization policy | 0..* | Analytical, automatic, implicit or finite-difference route actually eligible, with perturbation scales where needed. | Derivative meaning supplied by IC-39, not chosen implicitly by method availability. |

**Permitted variants:** One direct algorithm. Bounded sequence of same-problem retries. Qualified finite-difference derivative plan.

**Invariants:** Changing an algorithm may preserve physical problem identity while changing execution evidence. Dropping an allowed phase is a physical restriction even when initiated to improve convergence. Numerical bounds do not certify a model's scientific domain.

**Lifecycle and mutability:** Revise numerical evidence on policy changes; P10 reassesses affected results/branches without assuming all such changes alter physical definitions.

**Related concepts:** [IC-35](#ic-35), [IC-39](#ic-39), [IC-57](#ic-57).

**Valid interpretation:** Tightening termination tolerance updates execution/acceptance evidence while the original material model remains identifiable.

**Invalid interpretation:** Reducing the number of permitted phases is recorded only as a solver-tolerance tweak.

**Requirement basis:** [FR-CFG-05](thermodynamics_functional_requirements_v0_1.md#fr-cfg-05), [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05), [FR-LIF-03](thermodynamics_functional_requirements_v0_1.md#fr-lif-03). **Global rules:** SI-13, SI-21.

### P09 — Result qualification and coverage evidence

<a id="ic-59"></a>
#### IC-59. Check result and evidence evaluation

**Owner:** P09. **Kind:** assessment. **Step-5 product:** A14.

The evidence-based pass/fail/not-assessed assessment of one declared check against an original problem and candidate.

**Identity:** Check-result identity fixes obligation, candidate, evidence inputs and assessment revision; it does not rewrite the tolerance or original target.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Obligation and subject | 1 | The exact check criterion and candidate/original problem assessed. | Original target and captured context. |
| Evidence inputs | 1..* | Candidate values, reevaluations, process residuals, sources or an explicit unavailable-evidence record. | Same-model versus independent provenance stated. |
| Measured residual and scale | 0..* | Absolute residual, scale, normalized value, units and tolerance applied. | Unavailable residual is not zero; cancellation-safe scale policy retained. |
| Conclusion and applicability | 1 | Pass, fail, not assessed or not applicable with reason; mandatory role inherited from obligation. | No inference of physical validity beyond check scope. |
| Evidence dependence | 1 | What depends on the same model/data/solver and what is independent. | Reevaluation may check consistency without validating model fidelity. |

**Permitted variants:** Physical residual check. Authority preservation check. Structural identity/context comparison.

**Invariants:** Required-check absence blocks corresponding acceptance scope. Provider convergence is evidence, not an automatic passed target check. A check based on missing element coefficients cannot certify elemental conservation.

**Lifecycle and mutability:** Append immutable assessment; later evidence leads to another assessment retaining originals.

**Related concepts:** [IC-10](#ic-10), [IC-26](#ic-26), [IC-41](#ic-41), [IC-56](#ic-56).

**Valid interpretation:** The original PH target is compared against recomputed enthalpy with recorded model provenance.

**Invalid interpretation:** Agreement of two evaluations sharing the same model is labeled independent experimental validation.

**Requirement basis:** [FR-PRP-06](thermodynamics_functional_requirements_v0_1.md#fr-prp-06), [FR-CHM-08](thermodynamics_functional_requirements_v0_1.md#fr-chm-08), [FR-RES-02](thermodynamics_functional_requirements_v0_1.md#fr-res-02), [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03), [FR-RES-08](thermodynamics_functional_requirements_v0_1.md#fr-res-08). **Global rules:** SI-10, SI-23, SI-24. **Worked examples:** SE-01, SE-04, SE-05, SE-12, SE-13, SE-15.

<a id="ic-60"></a>
#### IC-60. Qualified result and acceptance assessment

**Owner:** P09. **Kind:** assessment. **Step-5 product:** A14.

What a candidate establishes for a specified physical problem and completion scope after required checks, independently of whether that result is current.

**Identity:** Assessment identity fixes candidate set, original problem(s), scope, check obligations and evidence revision.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Candidate and problem set | 1..* | Consistent candidate values and the exact questions they answer. | No unqualified mixture of model revisions. |
| Required scope and check set | 1 | Property/local/unit/flowsheet acceptance scope and all applicable mandatory/optional checks. | Exported scope descriptor; process criteria supplied by coordinator. |
| Outcome and completeness | 1 | Accepted for stated scope, rejected or unassessed; partial-property status preserved. | Outcome cause and missing/failed evidence explicit. |
| Independent quality dimensions | 1 | Numerical completion, constraint satisfaction, stability scope, scientific applicability, provenance completeness and validation evidence. | Currentness/publication are supplied facts from P10, not changed historical judgments. |
| Additional evidence needs | 0..* | Bounded requests for required missing evaluations under original conventions. | No permission to change targets or physics. |
| Lineage | 1 | Trace to definitions, inputs and actual execution evidence. | Reproducibility limitations retained. |

**Permitted variants:** Checked local state only. Accepted coherent unit product set. Rejected candidate retaining diagnostic value.

**Invariants:** Local acceptance does not imply unit/flowsheet convergence. Historically valid does not mean current. Accepted under an authorized approximation must retain that approximation label.

**Lifecycle and mutability:** P09 publishes assessments as evidence; P10 decides whether a referenced assessment may bind the current state.

**Related concepts:** [IC-34](#ic-34), [IC-35](#ic-35), [IC-41](#ic-41), [IC-56](#ic-56), [IC-59](#ic-59), [IC-62](#ic-62), [IC-63](#ic-63).

**Valid interpretation:** A revision-A result remains scientifically assessed for A after revision B becomes current.

**Invalid interpretation:** A valid A result bypasses the current revision guard and overwrites B.

**Requirement basis:** [FR-CHM-08](thermodynamics_functional_requirements_v0_1.md#fr-chm-08), [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08), [FR-RES-02](thermodynamics_functional_requirements_v0_1.md#fr-res-02), [FR-RES-04](thermodynamics_functional_requirements_v0_1.md#fr-res-04), [FR-RES-06](thermodynamics_functional_requirements_v0_1.md#fr-res-06). **Global rules:** SI-23, SI-24. **Worked examples:** SE-01, SE-13, SE-14, SE-15.

<a id="ic-61"></a>
#### IC-61. Coverage and validation claim

**Owner:** P09. **Kind:** assessment. **Step-5 product:** A17.

A bounded statement of representability, execution, implementation conformance or independent physical validation for a particular scenario/configuration.

**Identity:** Claim identity fixes subject scenario/profile, package/realization, envelope, evidence kind and evidence revision. Evidence dimensions are independent, not automatic promotions.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Claim subject | 1 | Scenario and exact material/model/data/operation/phase/chemistry/formulation scope. Concrete realization/build is required for execution claims, but may be unassigned for a representation-only review. | Specified boundary/envelope. |
| Evidence dimensions | 1..* | Representable, executable, implementation-conformant, physically consistent, reference-validated or reproducible, each with pass/fail/unassessed status. | No inference from a catalog link or documentation claim. |
| Witness records | 0..* | Executed fixture, review or reference comparison and actual manifest; absent witnesses remain explicit. | Predeclared tolerances, error/uncertainty budget and independence. |
| Limitations and gaps | 1 | Unassessed combinations, invalidated evidence and unmet P1/P2 delivery obligations. | Provider unsupported does not close program scope. |

**Permitted variants:** Representability review record. Pinned executed numerical fixture. Independent bounded data comparison.

**Invariants:** A source method or API argument is not an executed capability witness. Passing synthetic contract arithmetic cannot validate a real-fluid model. Agreement between providers sharing methods/data does not alone establish independent validation.

**Lifecycle and mutability:** Add/retire claims with evidence; retain historic witness conditions and never expand scope silently.

**Related concepts:** [IC-10](#ic-10), [IC-20](#ic-20), [IC-53](#ic-53), [IC-59](#ic-59), [IC-69](#ic-69).

**Valid interpretation:** A TP fixture adds evidence only for its declared material/model/conditions, not arbitrary reactive PH calculations.

**Invalid interpretation:** All 34 scenarios have mappings, so the simulator is advertised as validated for them.

**Requirement basis:** [FR-GOV-01](thermodynamics_functional_requirements_v0_1.md#fr-gov-01), [FR-GOV-05](thermodynamics_functional_requirements_v0_1.md#fr-gov-05), [FR-RES-06](thermodynamics_functional_requirements_v0_1.md#fr-res-06), [FR-RES-08](thermodynamics_functional_requirements_v0_1.md#fr-res-08), [FR-LIF-07](thermodynamics_functional_requirements_v0_1.md#fr-lif-07).

<a id="ic-62"></a>
#### IC-62. Result lineage and attribution

**Owner:** P09. **Kind:** evidence. **Step-5 product:** A14.

The dependency and provenance record explaining what physical problem, data and implementation produced a result, including requested-versus-used differences.

**Identity:** Lineage identity is tied to exact result/assessment evidence. It references immutable subjects rather than whatever happens to be current.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Definition and input references | 1..* | Material/representation, data, chemistry, package, binding/constraint descriptors and supplied state revisions actually used. | Exact revisions; unknown provenance explicit. |
| Implementation and execution references | 1..* | Realization, attempt, numerical policy, consequential seeds/branch and candidate evidence. | Actual build/binding/data manifest. |
| Requested-versus-used distinctions | 1 | All relevant deviations, estimated/substituted data and alternate-problem linkage. | No retrospective relabeling under current defaults. |
| Assessment and reproduction links | 0..* | Checks and validation/reproduction evidence attributed to this exact result. | Independent data distinguished from reused calculations. |
| Completeness of attribution | 1 | Complete for stated claim, limited or unassessed, with missing required facts. | Missing lineage lowers the claim rather than inventing metadata. |

**Permitted variants:** Fully visible model/data lineage. Pinned whole-provider bundle with limited internals.

**Invariants:** A provider upgrade cannot retroactively become the producer of old results. A reduced representation keeps its information-loss/assumption ancestry.

**Lifecycle and mutability:** Append evidence; preserve original producer/configuration association indefinitely for retained history.

**Related concepts:** [IC-03](#ic-03), [IC-14](#ic-14), [IC-20](#ic-20), [IC-28](#ic-28), [IC-35](#ic-35), [IC-53](#ic-53), [IC-54](#ic-54), [IC-56](#ic-56), [IC-59](#ic-59), [IC-61](#ic-61).

**Valid interpretation:** A downstream duty retains the fact that its interaction parameter was estimated and later approved.

**Invalid interpretation:** An old result is exported with today's database version in its provenance.

**Requirement basis:** [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07), [FR-LIF-04](thermodynamics_functional_requirements_v0_1.md#fr-lif-04). **Global rules:** SI-28.

<a id="ic-63"></a>
#### IC-63. Operation outcome and diagnostic statement

**Owner:** P09. **Kind:** definition. **Step-5 product:** A14.

The vocabulary and structure for reporting known outcomes, incomplete outputs and causes. Providers instantiate it; P09 owns its interpretation.

**Identity:** Statement identity identifies affected request/attempt/property/scope and source. Cause chains and per-property statements may coexist.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Affected subject and stage | 1 | Operation, property, location, attempt or required check and where failure/limitation occurred. | Precise scope, not only a generic exception text. |
| Outcome category | 1 | Use B4 categories: unknown identity, inconsistent/incomplete spec, authority conflict, unsupported/unassessed capability, missing data/dependency, incompatible references, domain, initializer, feasibility, branch, nonconvergence, checks, provider, cancellation/obsolete, optional property. | Semantic categories, not a required implementation enum. |
| Known cause and evidence | 1 | What is actually established; unknown native cause remains unknown; original provider detail retained where usable. | No infeasibility inference from a generic numerical exception. |
| Output qualification | 0..* | Which candidate outputs remain available, unaccepted or independently usable. | No absent quantity filled with zero. |
| Remedy or alternate problem link | 0..* | Missing prerequisites, required authorized change or separate alternate-problem result. | Recommendation is not automatic permission to change physics. |

**Permitted variants:** Structured early request rejection. Per-property unavailability. Nested numerical failure with retained cause chain.

**Invariants:** A failed optional property can coexist with primary-operation success. A provider failure, initialization failure and demonstrated infeasibility are different. Changed physical assumptions are retained as qualifications, not erased by a success label.

**Lifecycle and mutability:** Immutable statement per event/outcome; later recovery does not erase the original attempt's meaning.

**Related concepts:** [IC-26](#ic-26), [IC-35](#ic-35).

**Valid interpretation:** PH blocked by missing ideal caloric data is not reported as an impossible physical state.

**Invalid interpretation:** All native errors are converted into unsupported model or zero property.

**Requirement basis:** [FR-RUN-07](thermodynamics_functional_requirements_v0_1.md#fr-run-07), [FR-RES-01](thermodynamics_functional_requirements_v0_1.md#fr-res-01), [FR-RES-06](thermodynamics_functional_requirements_v0_1.md#fr-res-06). **Global rules:** SI-03. **Worked examples:** SE-14.

### P10 — Model revisions, publication and reconstruction

<a id="ic-64"></a>
#### IC-64. Revision and dependency context

**Owner:** P10. **Kind:** definition. **Step-5 product:** A16.

The exact semantic and realization references captured for a model, calculation or result and the dependency scope used for currency checks.

**Identity:** Context identity distinguishes entity lineage, immutable revision and snapshot of relevant dependencies. Content hash, semantic equivalence and current binding are different relations.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Revision identity and lineage | 1 | Stable entity lineage, exact immutable revision and optional parent/change ancestry. | Renaming/display changes separated from physical meaning. |
| Dependency set | 1..* | Material/data/method/chemistry/binding/specification and relevant implementation references captured together. | Exact revisions, not mutable aliases. |
| Dependency completeness | 1 | Exact relevant subset or conservative superset with known uncertainty. | Unknown impact cannot leave possibly stale results current. |
| Equivalence/compatibility evidence | 0..* | Explicit assessment for reuse under changed physical/presentation/numerical/implementation context. | Equal hashes are not proof of scientific equivalence. |
| Context kind | 1 | Approved model, trial proposal, historical snapshot or captured calculation context. | Trial may be complete without production approval. |

**Permitted variants:** Whole-model conservative snapshot. Qualified dependency-minimal capture. Provisional fitting context.

**Invariants:** A current pointer cannot stand in for the immutable context of a past result. Physical, numerical, implementation and presentation revisions can have different invalidation consequences. Revision ancestry is acyclic even though scientific/reference graphs may contain cycles.

**Lifecycle and mutability:** P10 commits new revisions; old references remain valid history. Selective invalidation is optional when conservative treatment is correct.

**Related concepts:** [IC-03](#ic-03), [IC-14](#ic-14), [IC-20](#ic-20), [IC-23](#ic-23), [IC-35](#ic-35).

**Valid interpretation:** A changed binary parameter invalidates all contexts that used its old revision.

**Invalid interpretation:** A cached state only checks component names and misses a changed reference convention.

**Requirement basis:** [FR-RUN-03](thermodynamics_functional_requirements_v0_1.md#fr-run-03), [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05), [FR-LIF-01](thermodynamics_functional_requirements_v0_1.md#fr-lif-01), [FR-LIF-03](thermodynamics_functional_requirements_v0_1.md#fr-lif-03), [FR-LIF-05](thermodynamics_functional_requirements_v0_1.md#fr-lif-05), [FR-LIF-06](thermodynamics_functional_requirements_v0_1.md#fr-lif-06). **Global rules:** SI-02, SI-25, SI-26, SI-28. **Worked examples:** SE-13, SE-15.

<a id="ic-65"></a>
#### IC-65. Coordinated change and impact proposal

**Owner:** P10. **Kind:** proposal. **Step-5 product:** A16.

A proposed semantic edit with explicit quantity preservation, domain validations and dependency effects before anything becomes current.

**Identity:** Change identity captures expected prior revision, proposed revisions, authorized intent and assessment results.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Expected prior context | 1 | The model/dependency revision against which the edit is proposed. | Stale edits require explicit reconcile/retry, not silent overwrite. |
| Proposed domain changes | 1..* | Validated proposals from material, data, chemistry, package, state or binding owners. | Content owners remain responsible for meaning. |
| Quantity-preservation policy | 0..1 | Required for material edits: preserve remaining component quantities, preserve total with authorized renormalization, or explicit other rule. | Removed material is not silently redistributed. |
| Dependency/impact assessment | 1 | Affected definitions, results, sessions and unresolved references, exact or conservative. | Physical versus numerical versus presentation effects distinguished. |
| Disposition and committed revision | 1 | Proposed, rejected, pending reconciliation or committed with exact new context. | No partial live adoption of an inconsistent coordinated set. |

**Permitted variants:** Composition patch. Assay-generated cut adoption. Provider upgrade or reference-convention change. Presentation-only edit.

**Invariants:** Removing an active reaction species cannot leave that reaction ready without repair. A characterization proposal may require material and parameter updates to be adopted together. Every interface uses the same edit semantics, not construction-history heuristics.

**Lifecycle and mutability:** Prepare -> domain-check/impact -> revision-checked commit or explicit rejection; undo is a new controlled restoration, not erasure of history.

**Related concepts:** [IC-03](#ic-03), [IC-14](#ic-14), [IC-20](#ic-20), [IC-27](#ic-27), [IC-45](#ic-45), [IC-49](#ic-49), [IC-64](#ic-64).

**Valid interpretation:** Removing A while preserving B=3 kg/s changes totals explicitly and invalidates affected state calculations.

**Invalid interpretation:** One editor preserves total and another preserves component flows without exposing the difference.

**Requirement basis:** [FR-MAT-05](thermodynamics_functional_requirements_v0_1.md#fr-mat-05), [FR-MAT-06](thermodynamics_functional_requirements_v0_1.md#fr-mat-06), [FR-DAT-03](thermodynamics_functional_requirements_v0_1.md#fr-dat-03), [FR-LIF-01](thermodynamics_functional_requirements_v0_1.md#fr-lif-01), [FR-LIF-02](thermodynamics_functional_requirements_v0_1.md#fr-lif-02), [FR-LIF-03](thermodynamics_functional_requirements_v0_1.md#fr-lif-03), [FR-LIF-06](thermodynamics_functional_requirements_v0_1.md#fr-lif-06), [FR-LIF-08](thermodynamics_functional_requirements_v0_1.md#fr-lif-08). **Global rules:** SI-12. **Worked examples:** SE-11.

<a id="ic-66"></a>
#### IC-66. Publication proposal and decision

**Owner:** P10. **Kind:** publication. **Step-5 product:** A15.

The coordinated decision to adopt a qualified result set for a current location/scope under matching revision and live run authority.

**Identity:** Proposal/decision identity references exact qualification, target binding slots, expected revision and run token. Rejected proposals remain attributable.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Qualified result set | 1..* | Accepted-for-scope assessments and corresponding candidate snapshots. | Related outputs form one coherent proposed set. |
| Target location/scope | 1..* | Where the result is proposed as current and whether it publishes constraints, allocation, state or whole-scope completion. | No implicit promotion from unit constraints to completed state. |
| Expected context and run authority | 1 | Matching revision/dependency set and permitted noncancelled run/supersession context. | Checked at same logical commit point as binding update. |
| Decision | 1 | Committed, stale, cancelled, superseded, scope-mismatched or insufficiently qualified with reasons. | Numerical validity is not redefined by publication rejection. |
| Affected current bindings | 0..* | Bindings atomically changed on success; none changed for a rejected coordinated publication. | Logical atomicity does not prescribe storage technology. |

**Permitted variants:** Publish constraints awaiting completion. Publish local accepted state. Publish a coherent unit or flowsheet result set.

**Invariants:** A check made before lengthy qualification must be revalidated at current-binding commit. No partial replacement of related unit outputs while claiming the whole unit is completed. An uninterruptible native call does not authorize publication after cancellation.

**Lifecycle and mutability:** P07 proposes scope, P09 supplies qualification, P10 commits or rejects currentness atomically at the logical boundary.

**Related concepts:** [IC-48](#ic-48), [IC-52](#ic-52), [IC-60](#ic-60), [IC-64](#ic-64), [IC-67](#ic-67), [IC-70](#ic-70).

**Valid interpretation:** A late A result is kept as history after B is committed but does not become B's current state.

**Invalid interpretation:** A cancelled run overwrites the published state because the provider could not interrupt immediately.

**Requirement basis:** [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06), [FR-RES-04](thermodynamics_functional_requirements_v0_1.md#fr-res-04), [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05), [FR-LIF-08](thermodynamics_functional_requirements_v0_1.md#fr-lif-08). **Global rules:** SI-25. **Worked examples:** SE-01, SE-13.

<a id="ic-67"></a>
#### IC-67. Current-result binding

**Owner:** P10. **Kind:** publication. **Step-5 product:** A15.

The authoritative association between a current model/location/scope and an accepted published result. It is a reference with guards, not a mutable copy of every result field.

**Identity:** Binding slot identity is model branch/location/publication kind/scope; each update has a revision and decision. At most one active result set per slot, while different scopes can coexist explicitly.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Target slot and scope | 1 | Current location/model branch and completion/publication kind. | Local versus unit/flowsheet bindings not silently interchangeable. |
| Qualified result reference | 1 | Exact result set admitted for this slot. | Immutable result/assessment references. |
| Current context and authority | 1 | Model/dependency revision and run authority under which this binding was committed. | Change or cancellation effects handled through explicit lifecycle rules. |
| Publication decision | 1 | The successful guarded decision creating the association. | No direct native-provider writes. |
| Currency state/history | 1 | Current, stale, withdrawn or historical association and reasons; prior links retained. | Historical validity remains separate. |

**Permitted variants:** Current local state. Current coherent unit product set. Stored historical binding no longer current.

**Invariants:** A stale binding cannot answer an unqualified current-state query. One physical result can be referenced by several derived reports without duplicating material. Updating a display does not alter the immutable result's producer or assumptions.

**Lifecycle and mutability:** Only P10 updates binding; change impact can invalidate it while preserving the referenced result.

**Related concepts:** [IC-48](#ic-48), [IC-52](#ic-52), [IC-60](#ic-60), [IC-64](#ic-64), [IC-66](#ic-66), [IC-70](#ic-70).

**Valid interpretation:** A parameter edit marks old output binding stale; the old state remains accessible with its original context.

**Invalid interpretation:** Historical arrays are relabeled as produced under the latest package because they remain visible in the UI.

**Requirement basis:** [FR-RES-04](thermodynamics_functional_requirements_v0_1.md#fr-res-04), [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05), [FR-RES-06](thermodynamics_functional_requirements_v0_1.md#fr-res-06). **Global rules:** SI-02, SI-23, SI-25. **Worked examples:** SE-13, SE-14, SE-15.

<a id="ic-68"></a>
#### IC-68. Semantic archive and dependency manifest

**Owner:** P10. **Kind:** definition. **Step-5 product:** A16.

The persisted material/method/problem context and reconstructable relationships required to inspect or reproduce a case, separate from optional runtime acceleration state.

**Identity:** Archive identity fixes format/semantic version, captured revisions, dependencies and content fingerprints. It is not a saved native memory address.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Semantic snapshots | 1..* | Material, data, chemistry, package, binding and input intent sufficient to rebuild the stated model. | Exact identities, revisions, map and convention meanings. |
| Recipe/evidence lineage | 0..* | Original assays, fitting/characterization recipes and approvals required by retained model claims. | Do not keep only generated cuts or solved arrays. |
| Provider/data dependencies | 1..* | Exact required engine/binding/artifact/data/version identities and availability/opacity limitations. | Unresolved dependencies may still permit inspection. |
| Stored results and lineage | 0..* | Historical states, qualifications, checks and provenance separate from model definitions. | No automatic currentness after loading. |
| Migration/reconstruction descriptors | 1 | Semantic format version, interpretation rules and unresolved extensions; caches marked optional/discardable. | No silent conversion of old physics to new defaults. |

**Permitted variants:** Self-contained visible semantic model plus dependencies. Pinned opaque provider bundle with reproducibility limits. Historical result archive with explicitly incomplete rebuild support.

**Invariants:** Saving cannot trigger hidden fitting or ordinary thermodynamic evaluation that changes approved definitions. Provider handles/caches are optional artifacts, not authoritative model meaning. A document may be inspectable while dependencies prevent execution.

**Lifecycle and mutability:** Create from captured snapshots; restore through IC-69 before any current result binding is granted.

**Related concepts:** [IC-03](#ic-03), [IC-10](#ic-10), [IC-14](#ic-14), [IC-16](#ic-16), [IC-17](#ic-17), [IC-20](#ic-20), [IC-28](#ic-28), [IC-45](#ic-45), [IC-49](#ic-49), [IC-53](#ic-53), [IC-60](#ic-60), [IC-62](#ic-62).

**Valid interpretation:** An assay case saves both original measurements and derived cuts/data.

**Invalid interpretation:** Only final species arrays and a native handle are saved, yet full reconstruction is claimed.

**Requirement basis:** [FR-GOV-03](thermodynamics_functional_requirements_v0_1.md#fr-gov-03), [FR-DAT-06](thermodynamics_functional_requirements_v0_1.md#fr-dat-06), [FR-LIF-04](thermodynamics_functional_requirements_v0_1.md#fr-lif-04), [FR-LIF-05](thermodynamics_functional_requirements_v0_1.md#fr-lif-05), [FR-LIF-07](thermodynamics_functional_requirements_v0_1.md#fr-lif-07). **Global rules:** SI-26.

<a id="ic-69"></a>
#### IC-69. Reconstruction, migration and reproduction record

**Owner:** P10. **Kind:** assessment. **Step-5 product:** A16.

What was restored and whether a rerun is a reproduction under recorded meaning or a comparison using changed model/dependencies.

**Identity:** Record identity ties source archive, target context, migration choices and actual attempt/evidence.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Source archive and target context | 1 | Saved semantic material and exact rebuilt revision. | Missing original dependencies remain explicit. |
| Reconstruction stages | 1..* | Inspection load, semantic rebuild, dependency resolution, readiness and restored-result authority assessed separately. | A loaded document is not necessarily ready or valid. |
| Migrations and differences | 0..* | Approved schema/semantic conversions, changed equations, parameter data, providers or unavailable resources. | Changes retained as differences, not hidden defaults. |
| Reproduction attempt and comparison | 0..* | Actual rerun, recorded manifest, tolerance and result difference analysis. | No bitwise guarantee; same-configuration versus changed-model comparison explicit. |
| Final qualification | 1 | Rebuilt/inspectable/blocked/reproduced/changed-model comparison or unassessed status with evidence. | P09 owns scientific/validation judgments incorporated by reference. |

**Permitted variants:** Inspection-only missing-provider load. Compatible reconstruction and bounded rerun. Explicit migration or model-upgrade comparison.

**Invariants:** Restoring numerical values into different equations does not restore original result authority. Unavailable original data prevents an unqualified exact-configuration reproduction claim.

**Lifecycle and mutability:** Append reconstruction decisions; establish new current bindings only after qualified matching checks.

**Related concepts:** [IC-54](#ic-54), [IC-60](#ic-60), [IC-61](#ic-61), [IC-64](#ic-64), [IC-65](#ic-65), [IC-68](#ic-68).

**Valid interpretation:** A changed parameter database yields a comparison record rather than an original-case reproduction claim.

**Invalid interpretation:** Applying a value-only snapshot to a rebuilt different formulation is declared lossless restoration.

**Requirement basis:** [FR-RES-08](thermodynamics_functional_requirements_v0_1.md#fr-res-08), [FR-LIF-05](thermodynamics_functional_requirements_v0_1.md#fr-lif-05), [FR-LIF-06](thermodynamics_functional_requirements_v0_1.md#fr-lif-06), [FR-LIF-07](thermodynamics_functional_requirements_v0_1.md#fr-lif-07). **Global rules:** SI-26.

<a id="ic-70"></a>
#### IC-70. Run authority and cancellation context

**Owner:** P10. **Kind:** definition. **Step-5 product:** A16.

The permission and captured context under which work may be published. Numerical execution can continue after this permission is revoked, but publication cannot.

**Identity:** Run identity is separate from model revision and individual numerical attempts; branch/scope/generation or equivalent logical authority distinguishes superseded work.

| Semantic content | Cardinality | Meaning and conditions | Units / basis |
| --- | --- | --- | --- |
| Captured model/dependency context | 1 | Original immutable context used by this run. | Exact revisions, including provisional branch if applicable. |
| Permitted publication scope | 1..* | Locations/scopes/result kinds that the run may propose. | Authority limits inherited by all child attempts. |
| Cancellation/supersession status | 1 | Active, cancelled, superseded, completed or explicitly revoked and reason. | Logical authority status, not proof a native thread stopped. |
| Parent/run relations | 0..1 | Parent campaign/coupled run, with explicit authority inheritance if nested. | No uncontrolled child publication after parent revocation. |
| Attempt references | 0..* | Work issued under this context. | Each attempt retains captured authority reference. |

**Permitted variants:** Interactive local solve. Whole-flowsheet solve. Parameter-fit campaign with no production-publication authority.

**Invariants:** Revision match alone is insufficient when a same-revision run has been cancelled or superseded. A new run can reuse a compatible session without reusing the old run's publication permission.

**Lifecycle and mutability:** Issue -> active -> complete/revoke/supersede; P10 guards current publication against present authority at commit.

**Related concepts:** [IC-48](#ic-48), [IC-52](#ic-52), [IC-54](#ic-54), [IC-64](#ic-64), [IC-70](#ic-70).

**Valid interpretation:** A late callback from a cancelled run is rejected even though no material parameter changed.

**Invalid interpretation:** No revision changed, so a cancelled result is published as current.

**Requirement basis:** [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06), [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05). **Global rules:** SI-25. **Worked examples:** SE-13.

## 5. Explicit relationship and cardinality catalog

Read each row as: one **source** can relate to the stated number of **targets**, and one target can be referenced by the stated number of sources. Repeated roles such as source/target representation are separate edges. A broad 0..* is a semantic capacity, not a concurrency or numerical capability promise. Conditions override no physical constraints. Containment, provenance and reference edges do not transfer content ownership.

| Relation | Source | Predicate | Target | Targets/source | Sources/target | Condition |
| --- | --- | --- | --- | --- | --- | --- |
| RL-001 | IC-02 | resolves to | IC-01 | 0..1 | 0..* | An assertion has one target only after unique resolution; unresolved candidates are not this resolved relation. |
| RL-002 | IC-03 | admits constituent revision | IC-01 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-003 | IC-04 | defines coordinates for | IC-03 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-004 | IC-05 | defines accounting on | IC-03 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-005 | IC-07 | has source representation | IC-03 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-006 | IC-07 | has target representation | IC-03 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-007 | IC-07 | preserves declared basis | IC-05 | 1..* | 0..* | Each claim states which basis and conditions; incomplete claims cannot imply all quantities conserved. |
| RL-008 | IC-08 | defines distribution on material-domain kind | IC-06 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-009 | IC-09 | defines loading for domain kind | IC-06 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-010 | IC-11 | is attributed to | IC-10 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-011 | IC-13 | uses parameter interpretation | IC-12 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-012 | IC-13 | binds subject roles to constituents | IC-01 | 0..* | 0..* | Applicable when subjects are constituent identities; group/site subjects retain separately named definitions rather than fictional constituent IDs. |
| RL-013 | IC-14 | selects parameter assertion | IC-13 | 0..* | 0..* | Zero visible assertions is allowed only with an attributable inseparable data bundle or a parameter-free selected method; unresolved needs remain explicit. |
| RL-014 | IC-14 | identifies source/data bundle | IC-10 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-015 | IC-16 | uses source evidence | IC-10 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-016 | IC-17 | was generated by recipe | IC-16 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-017 | IC-17 | proposes parameter assertion | IC-13 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-018 | IC-18 | requires parameter interpretation | IC-12 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-019 | IC-19 | selects intended method | IC-18 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-020 | IC-20 | admits material representation | IC-03 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-021 | IC-20 | uses resolved data snapshot | IC-14 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-022 | IC-20 | uses resolved method | IC-18 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-023 | IC-20 | uses convention set | IC-21 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-024 | IC-20 | uses physical eligibility policy | IC-22 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-025 | IC-20 | references chemistry | IC-45 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-026 | IC-20 | is qualified by compatibility assessment | IC-23 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-027 | IC-24 | assesses package for demand descriptor | IC-20 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-028 | IC-25 | uses authoritative accounting representation | IC-03 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-029 | IC-27 | describes one account | IC-25 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-030 | IC-27 | uses coordinate chart | IC-04 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-031 | IC-28 | describes one account snapshot | IC-25 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-032 | IC-28 | contains local phase/domain entries | IC-29 | 0..* | 1 | A phase-entry identity is local to exactly one state snapshot; reusing a physical interpretation across snapshots needs explicit correspondence. |
| RL-033 | IC-29 | instantiates domain kind | IC-06 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-034 | IC-30 | relates source phase entries | IC-29 | 0..* | 0..* | Zero sources means appearance; at least one side is nonempty. Cardinalities allow split, merge and ambiguity. |
| RL-035 | IC-30 | relates target phase entries | IC-29 | 0..* | 0..* | Zero targets means disappearance; at least one side is nonempty. No implicit identity preservation through coalescence. |
| RL-036 | IC-31 | views source account | IC-25 | 1..* | 0..* | Multiple-account aggregate is reporting only and requires explicit nonoverlapping selection; it does not create another physical inventory. |
| RL-037 | IC-31 | views selected phase/domain portions | IC-29 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-038 | IC-32 | instantiates distributed attribute | IC-08 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-039 | IC-32 | is associated with local account | IC-25 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-040 | IC-33 | uses loading basis | IC-09 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-041 | IC-33 | is associated with local account | IC-25 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-042 | IC-34 | declares observable demands | IC-38 | 0..* | 0..* | A contribution-only operation may instead demand an IC-40 description; complete operation requires at least one declared output kind. |
| RL-043 | IC-34 | declares postconditions | IC-41 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-044 | IC-35 | instantiates operation | IC-34 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-045 | IC-35 | captures package revision | IC-20 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-046 | IC-35 | captures input state | IC-28 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-047 | IC-35 | uses constraint set | IC-36 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-048 | IC-35 | uses authority allocation | IC-37 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-049 | IC-35 | uses effective restriction/branch record | IC-42 | 0..1 | 0..* | Required when any default is specialized or any constrained/metastable branch is requested; otherwise exact default policy is retained. |
| RL-050 | IC-39 | differentiates observable | IC-38 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-051 | IC-39 | uses independent coordinate chart | IC-04 | 0..* | 0..* | Required for composition derivatives; T/P/parameter-only derivatives still state their input coordinates explicitly. |
| RL-052 | IC-40 | binds variable and constraint meanings | IC-36 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-053 | IC-40 | declares derivative contributions | IC-39 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-054 | IC-43 | uses conserved basis | IC-05 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-055 | IC-44 | applies to reaction | IC-43 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-056 | IC-45 | admits species identity | IC-01 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-057 | IC-45 | uses reaction laws | IC-44 | 0..* | 0..* | Chemical-potential systems need not enumerate all independent reactions; data/conventions and allowed species still required. |
| RL-058 | IC-45 | uses chemical reconciliation | IC-47 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-059 | IC-45 | declares possible exchange requirement | IC-46 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-060 | IC-47 | has apparent representation | IC-03 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-061 | IC-47 | has true representation | IC-03 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-062 | IC-48 | has parent process location | IC-48 | 0..1 | 0..* | Root has no parent; location inheritance hierarchy is acyclic. |
| RL-063 | IC-49 | binds process location | IC-48 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-064 | IC-49 | references resolved package | IC-20 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-065 | IC-50 | declares participating locations | IC-48 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-066 | IC-50 | declares contribution demands | IC-40 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-067 | IC-51 | uses source binding | IC-49 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-068 | IC-51 | uses target binding | IC-49 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-069 | IC-51 | uses representation map | IC-07 | 0..* | 0..* | An identity map may be explicit; nonidentical representations require a qualified map, never an implicit common-name assumption. |
| RL-070 | IC-51 | creates target state problem | IC-35 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-071 | IC-52 | has parent completion scope | IC-52 | 0..1 | 0..* | Root has no parent; scope hierarchy is acyclic and does not imply upward convergence. |
| RL-072 | IC-53 | realizes operation contract | IC-34 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-073 | IC-54 | attempts physical problem | IC-35 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-074 | IC-54 | uses provider realization | IC-53 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-075 | IC-54 | uses numerical policy | IC-72 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-076 | IC-54 | uses initialization description | IC-57 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-077 | IC-54 | has retry parent | IC-54 | 0..1 | 0..* | Attempt ancestry is acyclic; physically changed attempts reference a distinct problem. |
| RL-078 | IC-55 | belongs to provider realization | IC-53 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-079 | IC-55 | supports active attempt | IC-54 | 0..* | 0..* | Multiplicity limited by actual sharing/isolation contract; 0..* is not a blanket concurrency guarantee. |
| RL-080 | IC-56 | was produced by attempt | IC-54 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-081 | IC-56 | contains normalized state snapshot | IC-28 | 0..* | 0..* | Failed/partial/contribution-only outcomes may have no completed state. |
| RL-082 | IC-56 | contains actual exchange | IC-71 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-083 | IC-57 | initializes target problem | IC-35 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-084 | IC-58 | binds native realization | IC-53 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-085 | IC-59 | assesses check obligation | IC-41 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-086 | IC-59 | uses candidate evidence | IC-56 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-087 | IC-60 | assesses candidate set | IC-56 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-088 | IC-60 | includes check assessments | IC-59 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-089 | IC-60 | retains lineage | IC-62 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-090 | IC-61 | qualifies scenario with check evidence | IC-59 | 0..* | 0..* | Absent execution/validation evidence remains unassessed; source/catalog review alone does not create a runtime pass. |
| RL-091 | IC-62 | traces physical problem | IC-35 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-092 | IC-65 | expects prior revision context | IC-64 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-093 | IC-65 | creates committed revision context | IC-64 | 0..1 | 0..* | Present only after successful coordinated commit; rejected proposal changes no current binding. |
| RL-094 | IC-66 | proposes qualified result set | IC-60 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-095 | IC-66 | checks captured revision | IC-64 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-096 | IC-66 | checks run authority | IC-70 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-097 | IC-66 | updates current binding | IC-67 | 0..* | 0..* | Zero for rejection. A successful coordinated group updates all its declared target slots at one logical commit point. |
| RL-098 | IC-67 | references qualified result | IC-60 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-099 | IC-67 | binds target location | IC-48 | 1..* | 0..* | A coherent unit/flowsheet publication slot may cover a group; at most one current accepted result set per declared slot and model branch. |
| RL-100 | IC-68 | captures model context | IC-64 | 1..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-101 | IC-68 | includes source lineage | IC-10 | 0..* | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-102 | IC-69 | reconstructs archive | IC-68 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-103 | IC-69 | records target context | IC-64 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-104 | IC-70 | captures revision context | IC-64 | 1 | 0..* | Applies to a complete record in the stated semantic role; unresolved drafts retain the missing fact explicitly. |
| RL-105 | IC-70 | has parent run authority | IC-70 | 0..1 | 0..* | Root has no parent; permission inheritance is explicit and ancestry acyclic. |
| RL-106 | IC-71 | has source account | IC-25 | 0..1 | 0..* | Absent only when an explicit external reservoir/boundary supplies the source. |
| RL-107 | IC-71 | has sink account | IC-25 | 0..1 | 0..* | Absent only when an explicit external reservoir/boundary is the sink; direction remains explicit. |
| RL-108 | IC-71 | instantiates chemical exchange requirement | IC-46 | 0..1 | 0..* | Required for constrained chemical reservoir exchange; ordinary process flow or heat-only transfer need not be chemical. |

## 6. Cross-concept invariants and invalid combinations

### SI-01. Identity is not a label

**Concepts:** [IC-01](#ic-01), [IC-02](#ic-02), [IC-03](#ic-03), [IC-58](#ic-58).

Resolve identity through scoped assertions and exact representation revisions; names, provider IDs and positions do not carry material authority.

**Positive witness:** A coordinate permutation leaves the same composition after all axes are mapped.

**Rejected witness:** Ambiguous aliases or unmatched derivative axes cannot be accepted.

### SI-02. Identity, revision, equivalence and currentness differ

**Concepts:** [IC-20](#ic-20), [IC-64](#ic-64), [IC-67](#ic-67).

Stable lineage, exact content revision, assessed semantic equivalence, runtime identity and current binding are distinct relations.

**Positive witness:** Two runtime sessions use the same immutable package revision.

**Rejected witness:** An old result is not relabeled under a new parameter revision.

### SI-03. Value state and role are orthogonal

**Concepts:** [IC-26](#ic-26), [IC-36](#ic-36), [IC-63](#ic-63).

Numeric availability is separate from specified/observed/guessed/calculated role, uncertainty and acceptance.

**Positive witness:** Known zero, unknown, failed and incipient values remain distinguishable.

**Rejected witness:** Missing viscosity does not become zero viscosity.

### SI-04. Quantities require subject and basis

**Concepts:** [IC-04](#ic-04), [IC-26](#ic-26), [IC-38](#ic-38).

Each quantity declares subject/domain, units, amount normalization, denominator, reference and tensor axes where necessary.

**Positive witness:** Molar enthalpy and total enthalpy cannot be confused when amount is absent.

**Rejected witness:** A vapor-fraction scalar without mass/mole basis cannot resolve a state.

### SI-05. Coordinate constraints define derivatives

**Concepts:** [IC-04](#ic-04), [IC-07](#ic-07), [IC-39](#ic-39).

Independent composition charts and held-fixed quantities are part of derivative identity. Coordinate permutations map all derivative axes; chart changes require the appropriate chain rule.

**Positive witness:** For q=x1+2x2+4x3 and x3=1−x1−x2, derivatives are −3 and −2.

**Rejected witness:** Returning 1 and 2 from an unconstrained three-coordinate derivative violates that chart.

### SI-06. One material account, no double counting

**Concepts:** [IC-25](#ic-25), [IC-27](#ic-27), [IC-29](#ic-29), [IC-31](#ic-31), [IC-47](#ic-47).

Only disjoint physical portions in a declared accounting partition are additive. Overall/apparent/true/reporting views are not extra portions.

**Positive witness:** Two liquid portions sum to their parent, while total-liquid view is excluded from the sum.

**Rejected witness:** Apparent salt and the ions representing it cannot both count toward physical mass.

### SI-07. Intensive, flow, inventory and reference amount differ

**Concepts:** [IC-25](#ic-25), [IC-26](#ic-26), [IC-71](#ic-71).

Account purpose and temporal basis are explicit; a reference-mole calculation is not physical holdup or a per-second rate.

**Positive witness:** A closed inventory has amounts,U,V and no mass-flow requirement.

**Rejected witness:** Multiplying an inventory by an implicit one-second duration is prohibited.

### SI-08. Zero does not determine composition

**Concepts:** [IC-25](#ic-25), [IC-27](#ic-27), [IC-28](#ic-28).

Zero flow can retain intended composition; empty physical inventory does not acquire unique intensive state by normalizing zeros.

**Positive witness:** A zero-flow feed keeps its specified fractions separately from zero component rates.

**Rejected witness:** Zero totals cannot be divided to claim a unique physical composition.

### SI-09. Phase category, presence and correspondence differ

**Concepts:** [IC-06](#ic-06), [IC-22](#ic-22), [IC-29](#ic-29), [IC-30](#ic-30), [IC-42](#ic-42).

Physical domains, candidates, present entries, incipient data, numerical placeholders and cross-snapshot identity are separate.

**Positive witness:** Two-to-one coalescence is recorded as merge, not false unique continuation.

**Rejected witness:** Sorting by density is not sufficient proof of persistent physical identity.

### SI-10. Phase scope is not stability proof

**Concepts:** [IC-22](#ic-22), [IC-35](#ic-35), [IC-42](#ic-42), [IC-56](#ic-56), [IC-59](#ic-59).

Results record requested physical candidates, structural slots and actually examined phase scope separately. Stability claims cannot exceed available evidence.

**Positive witness:** A restricted-liquid result retains its solid suppression.

**Rejected witness:** A two-phase fallback cannot certify an unrestricted three-phase problem.

### SI-11. Parameters are formula-qualified

**Concepts:** [IC-12](#ic-12), [IC-13](#ic-13), [IC-14](#ic-14).

A coefficient is interpreted by formula, roles, independent-variable units, convention and missing-value policy; source, estimate and substitute remain distinct.

**Positive witness:** Directed interactions retain their ordered subjects.

**Rejected witness:** Universal missing=zero or forced symmetry is rejected.

### SI-12. Preparation does not mutate evaluation inputs

**Concepts:** [IC-14](#ic-14), [IC-16](#ic-16), [IC-17](#ic-17), [IC-20](#ic-20), [IC-65](#ic-65).

Resolved snapshots are fixed for an attempt; fitting uses explicit trial snapshots and approved changes are coordinated commits.

**Positive witness:** A failed trial leaves approved data unchanged.

**Rejected witness:** Context binding or saving cannot silently fit new production parameters.

### SI-13. Scientific domains differ from numerical limits

**Concepts:** [IC-15](#ic-15), [IC-24](#ic-24), [IC-72](#ic-72).

Data range, model applicability, validated envelope and numerical search bounds are independently stated.

**Positive witness:** An allowed extrapolation is labeled as such.

**Rejected witness:** Being inside a root-finder bracket cannot establish model validity.

### SI-14. Reference transforms are scoped functions

**Concepts:** [IC-07](#ic-07), [IC-21](#ic-21), [IC-51](#ic-51).

Reference conversion has source/target conventions, material dependence, applicable domains and evidence. A universal scalar offset is not assumed.

**Positive witness:** For component offsets c, total-energy reference correction is sum(n_i c_i) on the declared basis.

**Rejected witness:** The remaining model difference is not renamed a reference correction or hidden heat.

### SI-15. Reaction energy reconciliation is consistent

**Concepts:** [IC-05](#ic-05), [IC-21](#ic-21), [IC-43](#ic-43), [IC-45](#ic-45), [IC-71](#ic-71).

For a declared species reference change H_b=H_a+c^T n, reaction-dependent changes c^T nu xi must be reconciled with the selected thermochemical/energy convention; arbitrary chemical-potential shifts are not silently permitted.

**Positive witness:** Equivalent formation-inclusive and correction-based bookkeeping has the same declared physical duty.

**Rejected witness:** Adding the full reaction heat again to formation-inclusive enthalpy fails consistency.

### SI-16. Open chemistry records actual exchanges

**Concepts:** [IC-46](#ic-46), [IC-47](#ic-47), [IC-71](#ic-71).

A chemical reservoir requirement is not permission; requested authority, actual exchange and its balance contribution are all explicit.

**Positive witness:** Reported titrant appears in element and relevant energy accounting.

**Rejected witness:** A closed-system solve cannot silently add gas or titrant.

### SI-17. Authority constrains computation

**Concepts:** [IC-34](#ic-34), [IC-35](#ic-35), [IC-37](#ic-37).

An operation changes only quantities permitted by the captured authority contract, regardless of what the numerical engine can do.

**Positive witness:** Density/root resolution preserves fixed species totals and phase allocation.

**Rejected witness:** Property refresh cannot re-equilibrate a caller-owned split.

### SI-18. Equilibrium completion is demand-specific

**Concepts:** [IC-34](#ic-34), [IC-38](#ic-38), [IC-41](#ic-41), [IC-56](#ic-56).

The operation promises explicit outputs and required checks; further property requests remain separate, and optional failures do not waive mandatory evidence.

**Positive witness:** Successful phase allocation can coexist with unavailable optional viscosity.

**Rejected witness:** An object returned by a flash is not assumed to contain all valid properties.

### SI-19. Effective properties require their closure

**Concepts:** [IC-18](#ic-18), [IC-31](#ic-31), [IC-38](#ic-38), [IC-39](#ic-39).

Effective transport and response properties name their closure and held-fixed/response semantics; additive balance totals remain algebraically defined.

**Positive witness:** Frozen-allocation Cp and relaxed-state dH/dT can be reported as distinct observables.

**Rejected witness:** Same units cannot justify replacing one with the other.

### SI-20. Equations are first-class information

**Concepts:** [IC-36](#ic-36), [IC-37](#ic-37), [IC-40](#ic-40).

Local variables, constraints, constitutive responsibility, scaling and derivative capabilities are explicitly mapped to the host problem; no symbolic provider is mandatory.

**Positive witness:** A value callback and an algebraic contribution each declare actual capability.

**Rejected witness:** A numerical zero does not prove structural sparsity; temporary fixes cannot remain hidden.

### SI-21. Numerical attempts preserve the original problem

**Concepts:** [IC-35](#ic-35), [IC-54](#ic-54), [IC-57](#ic-57), [IC-72](#ic-72).

Guesses, search bounds and retries are separate from the original physical target. Altered physics creates an attributable alternative problem.

**Positive witness:** Changing the algorithm retains the same original PH target.

**Rejected witness:** The returned enthalpy cannot replace the requested target for acceptance.

### SI-22. Provider session sharing requires evidence

**Concepts:** [IC-53](#ic-53), [IC-55](#ic-55), [IC-58](#ic-58).

A session has a compatibility key, complete-call isolation policy and recovery status; multiple handles/internal parallelism do not grant thread-safety claims.

**Positive witness:** A failed native context is quarantined before reuse.

**Rejected witness:** Stale buffers cannot become accepted outputs of a later request.

### SI-23. Nested scope and currentness are independent

**Concepts:** [IC-52](#ic-52), [IC-59](#ic-59), [IC-60](#ic-60), [IC-67](#ic-67).

Local acceptance may feed a run-local candidate view; parent scope requires its own complete residual/authority evidence. Currentness is a separate P10 fact.

**Positive witness:** Converged flash inside unconverged recycle stays local only.

**Rejected witness:** Two unrelated local successes cannot certify a coupled reactive solution.

### SI-24. Acceptance uses predeclared checks

**Concepts:** [IC-41](#ic-41), [IC-59](#ic-59), [IC-60](#ic-60).

Acceptance compares candidates to original conditions on justified bases with explicit tolerances/scales; unassessed mandatory checks block that scope.

**Positive witness:** A solver-success candidate with wrong PH residual is rejected.

**Rejected witness:** Changing tolerance after seeing a discrepancy cannot retroactively establish validation.

### SI-25. Publication is a guarded group decision

**Concepts:** [IC-64](#ic-64), [IC-66](#ic-66), [IC-67](#ic-67), [IC-70](#ic-70).

Revision/dependency, run/cancellation/supersession and scope checks occur at the same logical commit as current binding updates; related outputs form a coherent group.

**Positive witness:** Late A or cancelled same-A result remains history, not current.

**Rejected witness:** Half an exchanger result set cannot replace a previously accepted complete set as if fully solved.

### SI-26. Reconstruction preserves meaning before authority

**Concepts:** [IC-64](#ic-64), [IC-68](#ic-68), [IC-69](#ic-69).

Archives retain semantic recipes, exact data and references; inspection, readiness and restored-current authority are separate.

**Positive witness:** Missing provider permits inspection but blocks affected execution.

**Rejected witness:** Numerical arrays restored into changed equations are not a lossless model restoration.

### SI-27. Extensions have real semantic contracts

**Concepts:** [IC-06](#ic-06), [IC-08](#ic-08), [IC-09](#ic-09), [IC-32](#ic-32), [IC-33](#ic-33).

Surface and distribution material declares support/denominator, quantities, transformations and loss; no opaque extension bag establishes representability.

**Positive witness:** Mass-weighted distributions mix with material mass weights.

**Rejected witness:** A mean cannot uniquely reconstruct an omitted distribution.

### SI-28. Semantic relationships are not implementation imports

**Concepts:** [IC-20](#ic-20), [IC-24](#ic-24), [IC-53](#ic-53), [IC-62](#ic-62), [IC-64](#ic-64).

A relation/reference to another concept does not transfer ownership or require a reverse implementation dependency. Consumer-owned descriptors can carry evidence.

**Positive witness:** P03 readiness uses supplied capability facts without querying a live P08 session.

**Rejected witness:** The semantic reference graph is not falsely claimed acyclic like the package contract-construction graph.

## 7. Worked semantic examples

These examples instantiate the meanings and relationships without defining a runtime JSON shape or selecting an implementation. Exact synthetic arithmetic and a simple publication-permission conjunction were checked in this authoring run. That does not execute the inherited acceptance fixtures or establish provider, concurrency or real-fluid correctness.

### SE-01. A PH request is not the same record as its answer

**Scope links:** SC-02, SC-06. **Concepts:** [IC-20](#ic-20), [IC-25](#ic-25), [IC-26](#ic-26), [IC-28](#ic-28), [IC-35](#ic-35), [IC-36](#ic-36), [IC-37](#ic-37), [IC-41](#ic-41), [IC-54](#ic-54), [IC-56](#ic-56), [IC-59](#ic-59), [IC-60](#ic-60), [IC-66](#ic-66).

**Setup:** A synthetic single-domain material has mass-specific h(T)=2(T−300) kJ/kg. A valve-like outlet request supplies pressure 100 kPa, h target 50 kJ/kg, a 1 kg/s flow and fixed nonreacting composition. This is a semantic fixture, not a calibrated valve fluid.

**Information relation 1.** The package identifies the formula, datum T=300 K, data choice and admitted domain. The account purpose is flow; component quantities are not inventory.

**Information relation 2.** The input state separately contains fixed P and target h and an optional T=320 K guess. The original problem may solve T and any permitted phase allocation; it cannot change total component flow.

**Information relation 3.** Attempt A computes candidate T=325 K. The output assertion h=50 is calculated, not a replacement for the original target. The postcondition compares them under the original convention.

**Information relation 4.** The unit can publish the constraints as pending completion, or propose the qualified completed state. Those are distinct publication kinds.

**Consistency witness:** h(325)=50 kJ/kg. A candidate h=20 has absolute target residual 30 kJ/kg and cannot pass a sufficiently strict predeclared criterion. Source and candidate snapshots have distinct identities, while both reference the same immutable physical problem context.

**Rejected interpretation:** A numerical success flag or small solver step replaces the target-residual check; the 320 K guess becomes a fixed specification.

**Limit:** No provider algorithm or actual valve energy/phase behavior was tested. The arithmetic only checks the stated synthetic information relationships.

### SE-02. Two liquids, a report view and a coalescence event

**Scope links:** SC-10, SC-11. **Concepts:** [IC-25](#ic-25), [IC-27](#ic-27), [IC-28](#ic-28), [IC-29](#ic-29), [IC-30](#ic-30), [IC-31](#ic-31), [IC-50](#ic-50).

**Setup:** A synthetic binary material contains 10 mol with overall z=(0.5,0.5). Snapshot S1 contains L-a: 6 mol at x=(0.7,0.3) and L-b: 4 mol at x=(0.2,0.8).

**Information relation 1.** Both liquids are disjoint portions of one local account. They share liquid-domain semantics, but their IDs are S1-local. Total-liquid is an aggregate view, excluded from additive accounting.

**Information relation 2.** A provider returns the entries in reverse order on another solve. Native order is normalized before comparing physical descriptions. Unique correspondence requires declared compatible evidence, not slot index.

**Information relation 3.** An adjacent authored snapshot S2 has one 10 mol liquid at (0.5,0.5). A correspondence record may relate {L-a,L-b} to {L-c} as a merge or leave continuation unassessed.

**Information relation 4.** Light/heavy product assignment is a unit routing relation; it can change without redefining constituent identity or inventing a unique survivor.

**Consistency witness:** Component amounts: 6×0.7+4×0.2=5; 6×0.3+4×0.8=5 mol. Adding total-liquid 10 mol to its two portions would incorrectly yield 20 mol and is rejected.

**Rejected interpretation:** Slot one is always the same physical liquid or always the light outlet; a merged liquid inherits a unique predecessor without evidence.

**Limit:** No real phase diagram, stability or phase-matching algorithm is established by these invented snapshots.

### SE-03. Apparent and true species share one material account

**Scope links:** SC-18, SC-21, SC-30. **Concepts:** [IC-01](#ic-01), [IC-03](#ic-03), [IC-04](#ic-04), [IC-05](#ic-05), [IC-07](#ic-07), [IC-25](#ic-25), [IC-27](#ic-27), [IC-43](#ic-43), [IC-45](#ic-45), [IC-47](#ic-47), [IC-71](#ic-71).

**Setup:** Use a synthetic salt-like constituent EF with one unit each of conserved E and F and net charge zero. Its true representation is E+ and F−; one mole EF is mapped to one mole of each ion under the explicitly chosen complete-dissociation example.

**Information relation 1.** The feed representation has a one-mole molecular coordinate. The true representation has two species coordinates. Neither coordinate vector is the conserved quantity vector itself.

**Information relation 2.** The declared conserved basis is E-element amount, F-element amount and charge equivalents. Both descriptions map to (1,1,0).

**Information relation 3.** The material account is common to both descriptions. The apparent feed is a source/reporting view and is not added again to actual species inventory.

**Information relation 4.** In a more general chemistry problem, true species may require a speciation solve and the reverse apparent attribution may be nonunique. This simple map does not authorize assuming every electrolyte has the same fixed map.

**Consistency witness:** Conserved totals agree although molecular/species mole counts are 1 and 2. A net external addition would need its own IC-71 record; none is present in this closed example.

**Rejected interpretation:** Forcing molecular mole count conservation through dissociation, or adding apparent EF to the already represented ions.

**Limit:** Synthetic element/site accounting only. No real salt thermodynamics, pH or dissociation prediction is claimed.

### SE-04. Reference functions depend on composition and reaction

**Scope links:** SC-17, SC-18, SC-29. **Concepts:** [IC-05](#ic-05), [IC-07](#ic-07), [IC-21](#ic-21), [IC-43](#ic-43), [IC-51](#ic-51), [IC-59](#ic-59), [IC-71](#ic-71).

**Setup:** For a deliberately defined bookkeeping transformation, let H_b=H_a+10 n_A+40 n_B, with H in kJ and amounts in mol. All other physical quantities are unchanged by this declared transformation.

**Information relation 1.** The convention record contains the component-wise molar offsets, coordinate order, source/target scope and compatibility evidence. It is not a universal 25 kJ/mol shift.

**Information relation 2.** At z=(0.5,0.5) the molar reference difference is 25 kJ/mol; at z=(0.75,0.25) it is 17.5 kJ/mol.

**Information relation 3.** For a synthetic balanced A→B reaction on an equal conserved-mass basis, extent 0.4 mol changes the reference contribution by (40−10)×0.4=12 kJ. Under Q=ΔH+C bookkeeping, the corresponding correction C_b=C_a−12 kJ preserves the same declared Q.

**Information relation 4.** This bookkeeping relation does not permit arbitrary independent shifts of chemical potentials or equilibrium constants. Their standard-state and thermochemical definitions must be reconciled as part of one compatible chemical model.

**Consistency witness:** For nonreacting component balances Δn_i=0 over the closed boundary, component-wise constant offsets cancel. For reacting composition, a claimed offset that cancels only at one feed composition is not a universal energy correction.

**Rejected interpretation:** Use one scalar offset for every mixture; or add a full reaction heat on top of an already formation-inclusive material energy.

**Limit:** The example verifies algebra of a declared convention. Actual cross-provider transformations and reaction data still require profile-specific evidence.

### SE-05. Reference correction is not model-discrepancy correction

**Scope links:** SC-28, SC-29, SC-30. **Concepts:** [IC-21](#ic-21), [IC-35](#ic-35), [IC-36](#ic-36), [IC-49](#ic-49), [IC-51](#ic-51), [IC-59](#ic-59), [IC-71](#ic-71).

**Setup:** Use the earlier synthetic forms h_A=T−300 and h_B=1.2(T−300)+100 kJ/kg, with T in K and a declared known reference offset of 100 kJ/kg.

**Information relation 1.** At fixed T=350 K, h_A=50 and corrected h_B=60 kJ/kg. The remaining 10 kJ/kg is a model difference, not an additional reference offset.

**Information relation 2.** A T,P,composition-preserving boundary records that discrepancy after reference conversion. An energy-preserving P,corrected-H boundary solves corrected h_B=50, giving T=341.666… K.

**Information relation 3.** Heat-only exchange between separate materials needs internally consistent energy differences on each side, not a constituent map across the wall.

**Information relation 4.** A physical heat/work exchange is represented only if the process model actually specifies one; it is never invented to conceal incompatible simultaneous constraints.

**Consistency witness:** The two boundary policies are distinguishable problems. Corrected-H preservation gives 300+50/1.2 K, not 350 K.

**Rejected interpretation:** Claim fixed T=350 K and corrected H=50 simultaneously in package B without acknowledging inconsistency or an explicitly changed process problem.

**Limit:** No real-fluid package compatibility is demonstrated. This preserves the original synthetic fixture's meaning.

### SE-06. A derivative is an experiment, not a pair of names

**Scope links:** SC-04, SC-07, SC-26. **Concepts:** [IC-04](#ic-04), [IC-26](#ic-26), [IC-38](#ic-38), [IC-39](#ic-39), [IC-40](#ic-40), [IC-56](#ic-56).

**Setup:** Consider an authored two-portion response h(T,β)=(T−300)+100β kJ/kg with β(T)=(T−300)/100 on 300<T<400 K. It is an algebraic response example, not an asserted physical equilibrium model.

**Information relation 1.** At fixed β, ∂h/∂T=1 kJ/kg/K. Along the specified β(T) response, dh/dT=1+100×(1/100)=2 kJ/kg/K. Same output label and units do not make these the same derivative.

**Information relation 2.** The derivative request records output basis, independent input chart, held-fixed constraints, amount normalization, reaction/phase response and first-order or higher-order tensor axes.

**Information relation 3.** For a separate composition example q=x1+2x2+4x3 and chart x3=1−x1−x2, the requested constrained-coordinate derivatives are −3 and −2, not 1 and 2.

**Information relation 4.** The method used to calculate the derivative, such as analytical or finite difference, is execution evidence. A perturbation leaving the chart or changing the branch cannot validate another derivative silently.

**Consistency witness:** At T=325 K, β=0.25 and h=50; the two specified slopes remain 1 and 2. Composition-chart chain rule gives dq/dx1=1−4=−3, dq/dx2=2−4=−2.

**Rejected interpretation:** A weighted fixed-phase heat capacity is returned as a relaxed-equilibrium heat-capacity response solely because its units match.

**Limit:** These are checked algebraic distinctions, not claims about smoothness or derivatives of a real flash near a phase transition.

### SE-07. Mass-only empirical heating is a complete limited use

**Scope links:** SC-23, SC-31. **Concepts:** [IC-01](#ic-01), [IC-03](#ic-03), [IC-14](#ic-14), [IC-20](#ic-20), [IC-24](#ic-24), [IC-25](#ic-25), [IC-26](#ic-26), [IC-35](#ic-35), [IC-38](#ic-38), [IC-50](#ic-50).

**Setup:** A synthetic empirical solid has mass flow 2 kg/s and h(T)=2(T−300) kJ/kg. It has no justified molecular weight or elemental formula. Heat it from 300 to 325 K under a no-loss/no-work unit assumption.

**Information relation 1.** The representation permits mass and caloric operations; chemical and molar quantities remain unsupported unless independently supplied.

**Information relation 2.** The method/data snapshot identifies a specific mass-based enthalpy law and reference. Readiness is true for this heating calculation, not universally for every property.

**Information relation 3.** The unit heat duty is 2×[2×25]=100 kW. The material state and accepted limited result do not need a molecular coordinate vector.

**Information relation 4.** A subsequent molar enthalpy or fugacity request returns an explicit missing-basis/unsupported capability outcome and does not destroy the mass-based result.

**Consistency witness:** Δh=50 kJ/kg; duty=100 kJ/s. No atomic/molecular information is created by successful heat accounting.

**Rejected interpretation:** Assign a dummy molecular weight so the material fits a fluid interface and then report fictitious molar or elemental values.

**Limit:** Only the supplied synthetic law is used; no empirical material library or actual operating envelope is validated.

### SE-08. Distribution mixing and lossy reduction

**Scope links:** SC-32. **Concepts:** [IC-08](#ic-08), [IC-25](#ic-25), [IC-26](#ic-26), [IC-32](#ic-32), [IC-07](#ic-07), [IC-50](#ic-50).

**Setup:** Two portions use the same discrete size bins and mass-weighted normalized weights: 1 kg with [0.8,0.2], and 3 kg with [0.2,0.8].

**Information relation 1.** The attribute definition fixes bins, weighting measure and normalization; local realizations associate weights with actual material masses.

**Information relation 2.** The mixture bin masses are [1.4,2.6] kg, yielding weights [0.35,0.65] at 4 kg total.

**Information relation 3.** A permitted moment reduction retains the selected statistic, support/basis and the lost distribution information.

**Information relation 4.** If an operation selectively transfers one size class, its amount and distribution change together; it is not ordinary proportionate splitting.

**Consistency witness:** Bin masses sum to 4 kg and normalized weights sum to one. Equal averaging [0.5,0.5] does not represent this mass-weighted mixing.

**Rejected interpretation:** An equal-weight average or an invented full distribution reconstructed from a single mean is claimed lossless.

**Limit:** P3 information semantics only; no polymer or particle-population numerical model is implemented.

### SE-09. Surface loading and physical material exchange

**Scope links:** SC-27. **Concepts:** [IC-09](#ic-09), [IC-25](#ic-25), [IC-26](#ic-26), [IC-33](#ic-33), [IC-46](#ic-46), [IC-71](#ic-71).

**Setup:** A surface domain has total-loading convention 0.2 mol/kg dry sorbent and 5 kg dry sorbent, with 2 mol of the same accounted substance in a separate bulk portion.

**Information relation 1.** Initial stored surface amount is 1 mol. The surface and bulk portions are explicitly disjoint within the chosen total-amount accounting convention.

**Information relation 2.** An authorized 0.1 mol transfer from bulk to surface yields bulk=1.9 mol, surface=1.1 mol and loading=0.22 mol/kg at unchanged sorbent mass.

**Information relation 3.** A site-constrained model additionally records occupancy stoichiometry and capacity; an excess-adsorption convention would need its own amount relation rather than blindly reusing this total-loading formula.

**Information relation 4.** The denominator is a process-context quantity. Missing sorbent mass allows a loading assertion but not a unique actual adsorbed amount.

**Consistency witness:** Initial and final accounted total are both 3 mol. The transfer is an exchange relation, not a third material inventory.

**Rejected interpretation:** A loading change creates stored matter without a corresponding bulk/reservoir transfer.

**Limit:** P3 bookkeeping only; no adsorption isotherm, kinetics or membrane transport law is claimed.

### SE-10. Nonempty inventory, empty inventory and zero flow

**Scope links:** SC-33. **Concepts:** [IC-25](#ic-25), [IC-26](#ic-26), [IC-27](#ic-27), [IC-28](#ic-28), [IC-35](#ic-35), [IC-36](#ic-36), [IC-71](#ic-71).

**Setup:** Three descriptions are deliberately different: a nonempty inventory with known material, total U and V; a physically empty vessel; and a zero-flow feed with specified intended composition.

**Information relation 1.** The nonempty inventory can ask for T,P under a qualified U,V formulation without any flow field or duration. Material amount/composition is part of the specification.

**Information relation 2.** An empty physical inventory has no uniquely implied fluid intensive state. Wall temperature or vessel geometric volume may be known host quantities, but they are not inferred fluid properties of absent matter.

**Information relation 3.** A zero-flow feed can retain its intended fractions while all component flow rates are zero. Dividing rates by their zero total does not recreate those fractions.

**Information relation 4.** A provider reference-amount calculation may use one mole internally, but that basis never becomes actual vessel inventory without an explicit scale relation.

**Consistency witness:** Unknown composition from empty material remains unknown. Total U,V alone without justified material amount/composition does not establish a unique complete state.

**Rejected interpretation:** Fake kg/s makes an inventory look like a stream, or zero amount normalization invents a valid fluid state.

**Limit:** Actual UV numerical execution remains deferred for P3 unless separately claimed and tested.

### SE-11. Missing parameters, explicit zeros and immutable trials

**Scope links:** SC-08, SC-15. **Concepts:** [IC-12](#ic-12), [IC-13](#ic-13), [IC-14](#ic-14), [IC-16](#ic-16), [IC-17](#ic-17), [IC-20](#ic-20), [IC-54](#ic-54), [IC-65](#ic-65).

**Setup:** A directed binary parameter set contains an explicit selected A12=0 under one formula; A21 is required but missing. An estimator is authorized to propose A21 through a fitting campaign.

**Information relation 1.** The descriptor records direction, formula and units. A12's known zero and A21's missing state remain different assertions.

**Information relation 2.** Each trial has its own fixed parameter snapshot and captured package/problem. The production snapshot is not modified during evaluations.

**Information relation 3.** A successful proposal is reviewed and jointly adopted with updated readiness; a failed fit retains failed-trial evidence and the original missing prerequisite.

**Information relation 4.** A scientifically allowed substitute can be separately approved and labeled. That is not evidence that measured binary parameters were supplied.

**Consistency witness:** Source and approved snapshot hashes/revisions remain unchanged during ordinary evaluation or serialization. No symmetry relation is invented when only one directed parameter exists.

**Rejected interpretation:** Attaching a stream runs estimation, fills A21 with near-zero and leaves the same supposedly complete package identity.

**Limit:** This specifies preparation semantics; no regression algorithm, data source or physical parameter value is certified.

### SE-12. Equation contribution and temporary initialization fixing

**Scope links:** SC-07, SC-20, SC-26. **Concepts:** [IC-28](#ic-28), [IC-35](#ic-35), [IC-36](#ic-36), [IC-37](#ic-37), [IC-39](#ic-39), [IC-40](#ic-40), [IC-54](#ic-54), [IC-57](#ic-57), [IC-59](#ic-59).

**Setup:** A local stage contribution supplies r_h=h−h_model(T,P,x)=0. The unit owns material/energy balances and binds T,P,x,h to its global variables.

**Information relation 1.** The contribution declares its local coordinate chart, residual units, scaling and what it enforces. It does not duplicate the column's balance or own global variable values.

**Information relation 2.** Value-only callbacks, residual callbacks or symbolic contributions are alternative realization forms, each with explicit derivative availability.

**Information relation 3.** Initialization temporarily fixing T records the intervention. Final acceptance of the original problem requires releasing that fix or explicitly resolving it under the original constraints.

**Information relation 4.** A observed zero Jacobian entry at one state is numerical evidence, not proof that the corresponding structural dependency is absent.

**Consistency witness:** Residual scaling does not change the physical residual or its original unit convention. Derivative of r_h with respect to a local variable is distinguished from total sensitivity after solving the complete column.

**Rejected interpretation:** An initialization fix remains in force unnoticed, or a value-only provider is assumed to supply a Hessian.

**Limit:** No global equation IR, solver API, differentiation library or Rust type is selected.

### SE-13. Late results, same-revision cancellation and coherent publication

**Scope links:** SC-02, SC-06, SC-28. **Concepts:** [IC-52](#ic-52), [IC-54](#ic-54), [IC-59](#ic-59), [IC-60](#ic-60), [IC-64](#ic-64), [IC-66](#ic-66), [IC-67](#ic-67), [IC-70](#ic-70).

**Setup:** A run captures revision A and produces locally checked candidates. Before publication, either revision B is committed or the same-A run is cancelled/superseded.

**Information relation 1.** The candidate and P09 assessment retain the original A problem and check evidence. They can remain valid history even when not current.

**Information relation 2.** P10 tests expected revision/dependencies, run permission and publication scope at the logical commit that updates target current bindings.

**Information relation 3.** Revision equality alone is insufficient: an A result from a cancelled A run is still ineligible for current publication.

**Information relation 4.** For an exchanger or multi-outlet unit, the proposed group identifies all related results and the one captured scope; rejected groups leave prior bindings intact.

**Consistency witness:** Publish permission requires matching context AND active permission AND compatible accepted scope. A late A result under B, a cancelled A result under A, and a scope-mismatched group are rejected for currentness without rewriting scientific history.

**Rejected interpretation:** A prior revision check followed by unconditional output writes, or publishing half a unit's outputs from a different run.

**Limit:** Only guard semantics are defined; actual concurrency/atomicity implementation and fault-injection tests remain Step-7/implementation work.

### SE-14. Provider failure and optional property incompleteness

**Scope links:** SC-05, SC-13, SC-26. **Concepts:** [IC-24](#ic-24), [IC-34](#ic-34), [IC-41](#ic-41), [IC-53](#ic-53), [IC-55](#ic-55), [IC-56](#ic-56), [IC-60](#ic-60), [IC-63](#ic-63), [IC-67](#ic-67).

**Setup:** A candidate provides an admissible phase allocation and primary caloric result but fails an optional viscosity report. In a separate attempt, a native update fails and leaves working memory uncertain.

**Information relation 1.** The first outcome records per-property status. If mandatory checks pass, the primary scope can remain accepted while viscosity is explicitly unavailable. A rating mode requiring viscosity would be a different incomplete demand.

**Information relation 2.** The second outcome retains provider/stage diagnostics and candidate status, quarantines uncertain session state, and prevents retained native buffers from becoming a future result.

**Information relation 3.** Reconstruction of a session is distinct from reconstructing the semantic model. Both use exact compatible conventions, but neither alters the old accepted host state.

**Information relation 4.** A provider's opaque bundle can be used as a whole only with attributable artifacts and explicit visibility limits; internal fields need not be extracted to invent false certainty.

**Consistency witness:** Optional reporting failure does not satisfy a mandatory rating property. A fresh successful call must have fresh candidate provenance rather than stale buffer identity.

**Rejected interpretation:** All failures become zero properties, or any one optional property failure destroys an independently valid primary result.

**Limit:** Native recoverability/isolation remains build-specific and untested here.

### SE-15. A physical restriction is not a numerical starting guess

**Scope links:** SC-34. **Concepts:** [IC-20](#ic-20), [IC-22](#ic-22), [IC-28](#ic-28), [IC-29](#ic-29), [IC-30](#ic-30), [IC-35](#ic-35), [IC-37](#ic-37), [IC-42](#ic-42), [IC-56](#ic-56), [IC-59](#ic-59), [IC-60](#ic-60), [IC-64](#ic-64), [IC-67](#ic-67).

**Setup:** A hypothetical solution package admits a liquid and a named solid form. One local request explicitly suppresses the solid for a restricted-liquid calculation. A second request removes that restriction. No real solubility or metastable calculation is performed.

**Information relation 1.** The reusable resolved package retains the same methods and data. The first effective physical problem records the excluded solid, the reason for exclusion, the permitted liquid branch and the requested assessment scope.

**Information relation 2.** An initial guess containing no solid is a separate numerical fact. It does not by itself exclude the solid from the allowed phase search or establish metastability.

**Information relation 3.** Any candidate from the restricted request retains its restriction and can be assessed only against that request. Absence of a computed solid is not evidence that unrestricted solid stability was checked.

**Information relation 4.** Lifting the suppression creates a distinguishable physical problem and changes which results are eligible for current publication. The prior result retains its original meaning as restricted history; no numerical support for the new request is inferred.

**Consistency witness:** The base package identity can stay unchanged while the effective problem identity changes. An unavailable unrestricted stability check stays unassessed; a phase-constrained liquid root alone is not a certificate of metastability.

**Rejected interpretation:** Relabel the restricted result as an unrestricted stable equilibrium, or treat a zero-solid initial guess as permission to suppress precipitation.

**Limit:** This completes an authored information example for SC-34. Branch selection, metastability tests and actual numerical support remain unexecuted P3 work.

## 8. Functional-requirement information allocation

The normative wording, obligation classes, logical package ownership and original acceptance-test IDs remain unchanged. The primary information concept supplies a home for the requirement's main semantic contract; supporting concepts carry necessary relationships. A reference link does not prove implementation or numerical coverage.

| Requirement | Requirement title | Class | Step-5 owner | Primary concept | Supporting concepts |
| --- | --- | --- | --- | --- | --- |
| [FR-GOV-01](thermodynamics_functional_requirements_v0_1.md#fr-gov-01) | Qualify coverage at the scenario and configuration level | H | P09 | [IC-61](#ic-61) | [IC-53](#ic-53), [IC-24](#ic-24) |
| [FR-GOV-02](thermodynamics_functional_requirements_v0_1.md#fr-gov-02) | Describe the operation actually exposed by an adapter | H | P08 | [IC-53](#ic-53) | [IC-58](#ic-58) |
| [FR-GOV-03](thermodynamics_functional_requirements_v0_1.md#fr-gov-03) | Separate availability from model suitability | H | P08 | [IC-53](#ic-53) | [IC-68](#ic-68), [IC-24](#ic-24) |
| [FR-GOV-04](thermodynamics_functional_requirements_v0_1.md#fr-gov-04) | Admit cohesive packages and qualified submodels | H | P03 | [IC-20](#ic-20) | [IC-18](#ic-18), [IC-40](#ic-40), [IC-53](#ic-53) |
| [FR-GOV-05](thermodynamics_functional_requirements_v0_1.md#fr-gov-05) | Preserve declared scope and explicit coverage gaps | H | P09 | [IC-61](#ic-61) |  |
| [FR-MAT-01](thermodynamics_functional_requirements_v0_1.md#fr-mat-01) | Resolve identity without conflating labels | H | P01 | [IC-01](#ic-01) | [IC-02](#ic-02) |
| [FR-MAT-02](thermodynamics_functional_requirements_v0_1.md#fr-mat-02) | Declare the material representation and justified operations | H | P01 | [IC-03](#ic-03) | [IC-06](#ic-06), [IC-25](#ic-25) |
| [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03) | Keep intensive state, flow, and inventory distinct | H | P04 | [IC-25](#ic-25) | [IC-26](#ic-26), [IC-28](#ic-28), [IC-71](#ic-71) |
| [FR-MAT-04](thermodynamics_functional_requirements_v0_1.md#fr-mat-04) | Distinguish zero flow, empty material, and missing composition | H | P04 | [IC-25](#ic-25) | [IC-27](#ic-27), [IC-28](#ic-28) |
| [FR-MAT-05](thermodynamics_functional_requirements_v0_1.md#fr-mat-05) | Replace a complete composition explicitly | H | P04 | [IC-27](#ic-27) | [IC-26](#ic-26), [IC-65](#ic-65) |
| [FR-MAT-06](thermodynamics_functional_requirements_v0_1.md#fr-mat-06) | Patch component quantities with a preservation rule | H | P04 | [IC-27](#ic-27) | [IC-26](#ic-26), [IC-65](#ic-65) |
| [FR-MAT-07](thermodynamics_functional_requirements_v0_1.md#fr-mat-07) | Map composition coordinates consistently | H | P01 | [IC-04](#ic-04) | [IC-07](#ic-07), [IC-58](#ic-58) |
| [FR-MAT-08](thermodynamics_functional_requirements_v0_1.md#fr-mat-08) | Declare justified conserved quantities | H | P01 | [IC-05](#ic-05) | [IC-07](#ic-07), [IC-43](#ic-43) |
| [FR-DAT-01](thermodynamics_functional_requirements_v0_1.md#fr-dat-01) | Resolve parameter values with provenance | H | P02 | [IC-13](#ic-13) | [IC-14](#ic-14), [IC-10](#ic-10), [IC-11](#ic-11) |
| [FR-DAT-02](thermodynamics_functional_requirements_v0_1.md#fr-dat-02) | Interpret missing parameters according to the method | H | P02 | [IC-12](#ic-12) | [IC-13](#ic-13) |
| [FR-DAT-03](thermodynamics_functional_requirements_v0_1.md#fr-dat-03) | Authorize and record estimation or substitution | H | P02 | [IC-13](#ic-13) | [IC-17](#ic-17), [IC-65](#ic-65) |
| [FR-DAT-04](thermodynamics_functional_requirements_v0_1.md#fr-dat-04) | Keep fitted parameter trials separate from approved data | C | P02 | [IC-17](#ic-17) | [IC-16](#ic-16), [IC-14](#ic-14), [IC-54](#ic-54) |
| [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05) | Retain characterization inputs and generated materials | C | P02 | [IC-16](#ic-16) | [IC-17](#ic-17), [IC-01](#ic-01), [IC-03](#ic-03), [IC-14](#ic-14) |
| [FR-DAT-06](thermodynamics_functional_requirements_v0_1.md#fr-dat-06) | Prepare and serialize definitions without hidden evaluation effects | H | P02 | [IC-14](#ic-14) | [IC-16](#ic-16), [IC-17](#ic-17), [IC-68](#ic-68) |
| [FR-DAT-07](thermodynamics_functional_requirements_v0_1.md#fr-dat-07) | Preserve separate data and model validity information | H | P02 | [IC-15](#ic-15) | [IC-11](#ic-11), [IC-13](#ic-13) |
| [FR-CFG-01](thermodynamics_functional_requirements_v0_1.md#fr-cfg-01) | Identify a coherent configured package | H | P03 | [IC-20](#ic-20) | [IC-19](#ic-19), [IC-18](#ic-18), [IC-21](#ic-21) |
| [FR-CFG-02](thermodynamics_functional_requirements_v0_1.md#fr-cfg-02) | Validate composed-method compatibility | H | P03 | [IC-23](#ic-23) | [IC-18](#ic-18), [IC-21](#ic-21) |
| [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03) | Establish operation-specific readiness | H | P03 | [IC-24](#ic-24) | [IC-53](#ic-53), [IC-34](#ic-34), [IC-57](#ic-57) |
| [FR-CFG-04](thermodynamics_functional_requirements_v0_1.md#fr-cfg-04) | Qualify complete caloric behavior before energy-constrained use | H | P03 | [IC-21](#ic-21) | [IC-20](#ic-20), [IC-14](#ic-14), [IC-24](#ic-24) |
| [FR-CFG-05](thermodynamics_functional_requirements_v0_1.md#fr-cfg-05) | Separate physical policy from numerical policy | H | P03 | [IC-22](#ic-22) | [IC-42](#ic-42), [IC-72](#ic-72) |
| [FR-CFG-06](thermodynamics_functional_requirements_v0_1.md#fr-cfg-06) | Bind configurations to named material regions and local locations | H | P07 | [IC-49](#ic-49) | [IC-48](#ic-48) |
| [FR-CFG-07](thermodynamics_functional_requirements_v0_1.md#fr-cfg-07) | Declare candidate domains and phase/species eligibility | H | P03 | [IC-22](#ic-22) | [IC-06](#ic-06), [IC-29](#ic-29), [IC-42](#ic-42) |
| [FR-STA-01](thermodynamics_functional_requirements_v0_1.md#fr-sta-01) | Create a thermodynamic state at any required local location | H | P04 | [IC-28](#ic-28) | [IC-25](#ic-25), [IC-48](#ic-48) |
| [FR-STA-02](thermodynamics_functional_requirements_v0_1.md#fr-sta-02) | Check specification completeness and consistency | H | P05 | [IC-36](#ic-36) | [IC-35](#ic-35) |
| [FR-STA-03](thermodynamics_functional_requirements_v0_1.md#fr-sta-03) | Declare calculation authority per quantity family | H | P05 | [IC-37](#ic-37) | [IC-35](#ic-35) |
| [FR-STA-04](thermodynamics_functional_requirements_v0_1.md#fr-sta-04) | Preserve specified values separately from guesses and derived values | H | P04 | [IC-26](#ic-26) | [IC-28](#ic-28), [IC-57](#ic-57) |
| [FR-STA-05](thermodynamics_functional_requirements_v0_1.md#fr-sta-05) | Identify phase instances and aggregate views without positional meaning | H | P04 | [IC-29](#ic-29) | [IC-30](#ic-30), [IC-31](#ic-31) |
| [FR-STA-06](thermodynamics_functional_requirements_v0_1.md#fr-sta-06) | Qualify absent and incipient phase information | H | P04 | [IC-29](#ic-29) | [IC-30](#ic-30) |
| [FR-STA-07](thermodynamics_functional_requirements_v0_1.md#fr-sta-07) | Keep flow direction separate from physical composition | H | P04 | [IC-71](#ic-71) | [IC-26](#ic-26), [IC-27](#ic-27) |
| [FR-STA-08](thermodynamics_functional_requirements_v0_1.md#fr-sta-08) | Define reporting bases and reference conditions | H | P04 | [IC-31](#ic-31) | [IC-26](#ic-26), [IC-04](#ic-04) |
| [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01) | Resolve a supported TP equilibrium problem | C | P05 | [IC-35](#ic-35) | [IC-34](#ic-34), [IC-36](#ic-36), [IC-37](#ic-37) |
| [FR-EQL-02](thermodynamics_functional_requirements_v0_1.md#fr-eql-02) | Resolve PH with a verified enthalpy target | C | P05 | [IC-35](#ic-35) | [IC-36](#ic-36), [IC-41](#ic-41), [IC-21](#ic-21) |
| [FR-EQL-03](thermodynamics_functional_requirements_v0_1.md#fr-eql-03) | Resolve PS and distinguish reference from actual equipment state | C | P05 | [IC-35](#ic-35) | [IC-36](#ic-36), [IC-41](#ic-41), [IC-50](#ic-50) |
| [FR-EQL-04](thermodynamics_functional_requirements_v0_1.md#fr-eql-04) | Distinguish saturation endpoints from complete phase allocation | C | P05 | [IC-36](#ic-36) | [IC-35](#ic-35), [IC-29](#ic-29), [IC-38](#ic-38) |
| [FR-EQL-05](thermodynamics_functional_requirements_v0_1.md#fr-eql-05) | Return only the promised post-equilibrium properties | H | P05 | [IC-34](#ic-34) | [IC-38](#ic-38), [IC-56](#ic-56) |
| [FR-EQL-06](thermodynamics_functional_requirements_v0_1.md#fr-eql-06) | Qualify stability and searched phase scope | H | P05 | [IC-42](#ic-42) | [IC-41](#ic-41), [IC-29](#ic-29), [IC-56](#ic-56) |
| [FR-EQL-07](thermodynamics_functional_requirements_v0_1.md#fr-eql-07) | Identify branches and honor branch-selection policy | H | P05 | [IC-42](#ic-42) | [IC-30](#ic-30), [IC-57](#ic-57) |
| [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08) | Expose bounded multiple-liquid and solid-equilibrium coverage | C | P05 | [IC-34](#ic-34) | [IC-35](#ic-35), [IC-22](#ic-22), [IC-29](#ic-29), [IC-53](#ic-53) |
| [FR-PRP-01](thermodynamics_functional_requirements_v0_1.md#fr-prp-01) | Evaluate a supplied phase without unrequested redistribution | C | P05 | [IC-34](#ic-34) | [IC-37](#ic-37), [IC-28](#ic-28) |
| [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02) | Distinguish primary and optional property demands | H | P05 | [IC-34](#ic-34) | [IC-38](#ic-38), [IC-41](#ic-41), [IC-56](#ic-56) |
| [FR-PRP-03](thermodynamics_functional_requirements_v0_1.md#fr-prp-03) | Provide phase transport and phase-pair properties with explicit scope | C | P05 | [IC-38](#ic-38) | [IC-34](#ic-34), [IC-29](#ic-29) |
| [FR-PRP-04](thermodynamics_functional_requirements_v0_1.md#fr-prp-04) | Name the closure behind an effective multiphase property | C | P05 | [IC-38](#ic-38) | [IC-18](#ic-18), [IC-31](#ic-31) |
| [FR-PRP-05](thermodynamics_functional_requirements_v0_1.md#fr-prp-05) | Define derivative meaning before computation | H | P05 | [IC-39](#ic-39) | [IC-04](#ic-04), [IC-38](#ic-38) |
| [FR-PRP-06](thermodynamics_functional_requirements_v0_1.md#fr-prp-06) | Qualify derivative method, conditioning, and nonsmoothness | C | P05 | [IC-39](#ic-39) | [IC-40](#ic-40), [IC-56](#ic-56), [IC-59](#ic-59) |
| [FR-PRP-07](thermodynamics_functional_requirements_v0_1.md#fr-prp-07) | Keep caloric, standard-state, and chemical-potential conventions explicit | H | P03 | [IC-21](#ic-21) | [IC-18](#ic-18) |
| [FR-CHM-01](thermodynamics_functional_requirements_v0_1.md#fr-chm-01) | Declare reaction definitions independently of reactor closure | C | P06 | [IC-43](#ic-43) | [IC-44](#ic-44) |
| [FR-CHM-02](thermodynamics_functional_requirements_v0_1.md#fr-chm-02) | Honor strict specified-conversion feasibility | C | P06 | [IC-43](#ic-43) | [IC-44](#ic-44), [IC-36](#ic-36), [IC-41](#ic-41) |
| [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03) | Resolve chemical equilibrium under declared constraints | C | P05 | [IC-35](#ic-35) | [IC-45](#ic-45), [IC-43](#ic-43), [IC-46](#ic-46) |
| [FR-CHM-04](thermodynamics_functional_requirements_v0_1.md#fr-chm-04) | Preserve finite-rate and frozen-chemistry assumptions | C | P06 | [IC-45](#ic-45) | [IC-44](#ic-44), [IC-37](#ic-37) |
| [FR-CHM-05](thermodynamics_functional_requirements_v0_1.md#fr-chm-05) | Reconcile reaction and formation energy exactly once | H | P03 | [IC-21](#ic-21) | [IC-43](#ic-43), [IC-44](#ic-44), [IC-71](#ic-71) |
| [FR-CHM-06](thermodynamics_functional_requirements_v0_1.md#fr-chm-06) | Map apparent and true species without duplicate inventory | C | P06 | [IC-47](#ic-47) | [IC-07](#ic-07), [IC-25](#ic-25), [IC-05](#ic-05) |
| [FR-CHM-07](thermodynamics_functional_requirements_v0_1.md#fr-chm-07) | Account explicitly for chemically constrained external exchanges | C | P06 | [IC-46](#ic-46) | [IC-71](#ic-71), [IC-35](#ic-35) |
| [FR-CHM-08](thermodynamics_functional_requirements_v0_1.md#fr-chm-08) | Verify combined reaction and phase conditions | C | P09 | [IC-60](#ic-60) | [IC-59](#ic-59), [IC-41](#ic-41), [IC-45](#ic-45) |
| [FR-FLW-01](thermodynamics_functional_requirements_v0_1.md#fr-flw-01) | Expose the unit’s property demands and physical constraints | H | P07 | [IC-50](#ic-50) | [IC-48](#ic-48), [IC-34](#ic-34) |
| [FR-FLW-02](thermodynamics_functional_requirements_v0_1.md#fr-flw-02) | Preserve mixing and mechanical-splitting semantics | C | P07 | [IC-50](#ic-50) | [IC-71](#ic-71), [IC-25](#ic-25), [IC-27](#ic-27) |
| [FR-FLW-03](thermodynamics_functional_requirements_v0_1.md#fr-flw-03) | Separate internal phase state from product allocation | C | P07 | [IC-50](#ic-50) | [IC-29](#ic-29), [IC-30](#ic-30), [IC-71](#ic-71) |
| [FR-FLW-04](thermodynamics_functional_requirements_v0_1.md#fr-flw-04) | Couple disjoint material regions through heat | C | P07 | [IC-50](#ic-50) | [IC-21](#ic-21), [IC-71](#ic-71) |
| [FR-FLW-05](thermodynamics_functional_requirements_v0_1.md#fr-flw-05) | Translate a material across package boundaries under explicit constraints | C | P07 | [IC-51](#ic-51) | [IC-49](#ic-49), [IC-21](#ic-21), [IC-35](#ic-35) |
| [FR-FLW-06](thermodynamics_functional_requirements_v0_1.md#fr-flw-06) | Translate representations with declared conservation and information loss | C | P07 | [IC-51](#ic-51) | [IC-07](#ic-07), [IC-47](#ic-47) |
| [FR-FLW-07](thermodynamics_functional_requirements_v0_1.md#fr-flw-07) | Support equation-oriented participation without requiring symbolic providers | C | P07 | [IC-50](#ic-50) | [IC-40](#ic-40), [IC-36](#ic-36) |
| [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08) | Preserve bulk/interface separation in nonequilibrium contacting | C | P07 | [IC-50](#ic-50) | [IC-48](#ic-48), [IC-28](#ic-28), [IC-29](#ic-29), [IC-37](#ic-37) |
| [FR-RUN-01](thermodynamics_functional_requirements_v0_1.md#fr-run-01) | Qualify initialization independently of formulation support | H | P08 | [IC-57](#ic-57) | [IC-24](#ic-24), [IC-53](#ic-53) |
| [FR-RUN-02](thermodynamics_functional_requirements_v0_1.md#fr-run-02) | Preserve the original problem through initialization | H | P08 | [IC-57](#ic-57) | [IC-54](#ic-54), [IC-40](#ic-40) |
| [FR-RUN-03](thermodynamics_functional_requirements_v0_1.md#fr-run-03) | Bind reusable sessions to compatible configuration revisions | H | P08 | [IC-55](#ic-55) | [IC-53](#ic-53), [IC-64](#ic-64) |
| [FR-RUN-04](thermodynamics_functional_requirements_v0_1.md#fr-run-04) | Isolate complete calculation sequences across cases | H | P08 | [IC-55](#ic-55) | [IC-54](#ic-54), [IC-58](#ic-58) |
| [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05) | Separate same-problem retry from an alternate physical calculation | H | P08 | [IC-54](#ic-54) | [IC-72](#ic-72), [IC-35](#ic-35), [IC-42](#ic-42) |
| [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06) | Support cancellation without publishing interrupted trials | H | P08 | [IC-54](#ic-54) | [IC-55](#ic-55), [IC-70](#ic-70), [IC-66](#ic-66) |
| [FR-RUN-07](thermodynamics_functional_requirements_v0_1.md#fr-run-07) | Contain provider failures and qualify subsequent reuse | H | P08 | [IC-55](#ic-55) | [IC-56](#ic-56), [IC-63](#ic-63) |
| [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08) | Record nested convergence independently | H | P07 | [IC-52](#ic-52) | [IC-54](#ic-54), [IC-60](#ic-60) |
| [FR-RES-01](thermodynamics_functional_requirements_v0_1.md#fr-res-01) | Return structured, distinguishable operation outcomes | H | P09 | [IC-63](#ic-63) | [IC-56](#ic-56) |
| [FR-RES-02](thermodynamics_functional_requirements_v0_1.md#fr-res-02) | Adopt results only after declared acceptance checks | H | P09 | [IC-59](#ic-59) | [IC-60](#ic-60), [IC-41](#ic-41) |
| [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03) | Apply conservation and thermal checks on justified bases | H | P09 | [IC-59](#ic-59) | [IC-05](#ic-05), [IC-21](#ic-21), [IC-71](#ic-71) |
| [FR-RES-04](thermodynamics_functional_requirements_v0_1.md#fr-res-04) | Publish only the authorized result scope | H | P09 | [IC-60](#ic-60) | [IC-66](#ic-66), [IC-67](#ic-67), [IC-52](#ic-52) |
| [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05) | Protect accepted results from failed or obsolete work | H | P10 | [IC-66](#ic-66) | [IC-67](#ic-67), [IC-64](#ic-64), [IC-70](#ic-70) |
| [FR-RES-06](thermodynamics_functional_requirements_v0_1.md#fr-res-06) | Expose independent result-quality dimensions | H | P09 | [IC-60](#ic-60) | [IC-63](#ic-63), [IC-61](#ic-61), [IC-67](#ic-67) |
| [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07) | Retain result lineage and requested-versus-used assumptions | H | P09 | [IC-62](#ic-62) | [IC-10](#ic-10), [IC-14](#ic-14), [IC-54](#ic-54) |
| [FR-RES-08](thermodynamics_functional_requirements_v0_1.md#fr-res-08) | Distinguish numerical conformance from physical validation | H | P09 | [IC-61](#ic-61) | [IC-59](#ic-59), [IC-69](#ic-69) |
| [FR-LIF-01](thermodynamics_functional_requirements_v0_1.md#fr-lif-01) | Create a revision and impact assessment for consequential changes | H | P10 | [IC-64](#ic-64) | [IC-65](#ic-65) |
| [FR-LIF-02](thermodynamics_functional_requirements_v0_1.md#fr-lif-02) | Change a material slate with an explicit quantity/dependency policy | H | P10 | [IC-65](#ic-65) | [IC-03](#ic-03), [IC-27](#ic-27), [IC-45](#ic-45) |
| [FR-LIF-03](thermodynamics_functional_requirements_v0_1.md#fr-lif-03) | Separate presentation changes from physical and numerical changes | H | P10 | [IC-65](#ic-65) | [IC-64](#ic-64), [IC-72](#ic-72) |
| [FR-LIF-04](thermodynamics_functional_requirements_v0_1.md#fr-lif-04) | Serialize the semantic model and resolved data | H | P10 | [IC-68](#ic-68) | [IC-16](#ic-16), [IC-20](#ic-20), [IC-62](#ic-62) |
| [FR-LIF-05](thermodynamics_functional_requirements_v0_1.md#fr-lif-05) | Reconstruct definitions before restoring result authority | H | P10 | [IC-69](#ic-69) | [IC-68](#ic-68), [IC-64](#ic-64) |
| [FR-LIF-06](thermodynamics_functional_requirements_v0_1.md#fr-lif-06) | Restore snapshots without losing lineage or dependency consistency | H | P10 | [IC-69](#ic-69) | [IC-65](#ic-65), [IC-64](#ic-64) |
| [FR-LIF-07](thermodynamics_functional_requirements_v0_1.md#fr-lif-07) | Reproduce cases within declared tolerances and manifest limits | H | P10 | [IC-69](#ic-69) | [IC-68](#ic-68), [IC-61](#ic-61) |
| [FR-LIF-08](thermodynamics_functional_requirements_v0_1.md#fr-lif-08) | Apply the same change semantics through every interface | H | P10 | [IC-65](#ic-65) | [IC-66](#ic-66) |
| [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01) | Represent surface and selective-transfer domains meaningfully | R3 | P01 | [IC-09](#ic-09) | [IC-06](#ic-06), [IC-33](#ic-33), [IC-71](#ic-71) |
| [FR-EXT-02](thermodynamics_functional_requirements_v0_1.md#fr-ext-02) | Represent distributed material attributes and their reductions | R3 | P01 | [IC-08](#ic-08) | [IC-32](#ic-32), [IC-07](#ic-07) |
| [FR-EXT-03](thermodynamics_functional_requirements_v0_1.md#fr-ext-03) | Represent inventory-based state specifications without a flow workaround | R3 | P04 | [IC-25](#ic-25) | [IC-28](#ic-28), [IC-26](#ic-26), [IC-35](#ic-35) |
| [FR-EXT-04](thermodynamics_functional_requirements_v0_1.md#fr-ext-04) | Represent restricted-equilibrium and metastable requests explicitly | R3 | P05 | [IC-42](#ic-42) | [IC-35](#ic-35), [IC-30](#ic-30) |

## 9. Scenario and integrated-journey routes

All 34 scenarios retain their original profile. Distinctive concepts supplement the common package/state/problem/assessment/revision context; they do not waive applicable host-wide rules. A scenario without a separate worked example still has its original SA/AT acceptance contract and its explicit information route. P3 has concrete authored examples but no independent R review or numerical E/V promotion.

| Scenario / profile | Title | Distinctive information concepts | Examples |
| --- | --- | --- | --- |
| SC-01 / P1 | Blending and splitting without reaction | IC-05, IC-07, IC-21, IC-25, IC-26, IC-27, IC-43, IC-50, IC-59, IC-65, IC-71 | Original scenario contract retained |
| SC-02 / P1 | Sensible heating and heat exchange | IC-14, IC-20, IC-21, IC-24, IC-29, IC-34, IC-35, IC-36, IC-38, IC-41, IC-48, IC-50, IC-56, IC-71 | SE-01, SE-13 |
| SC-03 / P1 | Liquid pumping and pressure-loss calculations | IC-24, IC-28, IC-29, IC-34, IC-37, IC-38, IC-41, IC-42, IC-48, IC-50, IC-53, IC-56, IC-57 | Original scenario contract retained |
| SC-04 / P1 | Gas compression, expansion and intercooling | IC-04, IC-14, IC-20, IC-21, IC-24, IC-26, IC-28, IC-34, IC-35, IC-36, IC-38, IC-39, IC-41, IC-48, IC-50, IC-57 | SE-06 |
| SC-05 / P1 | Cooling and vapor–liquid separation | IC-05, IC-21, IC-29, IC-30, IC-31, IC-34, IC-35, IC-36, IC-37, IC-41, IC-42, IC-50, IC-56, IC-59, IC-71 | SE-14 |
| SC-06 / P1 | Pressure reduction with flashing | IC-21, IC-24, IC-35, IC-36, IC-37, IC-41, IC-52, IC-53, IC-57, IC-59, IC-60, IC-64, IC-66, IC-67, IC-70 | SE-01, SE-13 |
| SC-07 / P1 | Conventional equilibrium-stage distillation | IC-25, IC-26, IC-28, IC-34, IC-36, IC-37, IC-40, IC-48, IC-50, IC-52, IC-54, IC-57, IC-60, IC-66, IC-67 | SE-06, SE-12 |
| SC-08 / P1 | Nonideal-liquid separation and azeotropic behavior | IC-12, IC-13, IC-14, IC-17, IC-18, IC-20, IC-21, IC-23, IC-24, IC-30, IC-35, IC-42, IC-54, IC-57, IC-65, IC-72 | SE-11 |
| SC-09 / P1 | Physical absorption, humidification and gas dissolution | IC-04, IC-06, IC-18, IC-21, IC-22, IC-23, IC-26, IC-28, IC-29, IC-31, IC-34, IC-37, IC-42, IC-48, IC-50 | Original scenario contract retained |
| SC-10 / P2 | Liquid–liquid extraction and decanting | IC-22, IC-29, IC-30, IC-31, IC-34, IC-35, IC-38, IC-50, IC-53, IC-71 | SE-02 |
| SC-11 / P2 | Vapor–liquid–liquid separation | IC-05, IC-06, IC-21, IC-22, IC-29, IC-34, IC-35, IC-36, IC-37, IC-41, IC-42, IC-53, IC-56, IC-59, IC-71 | SE-02 |
| SC-12 / P1 | Water and steam through saturation | IC-04, IC-21, IC-26, IC-29, IC-31, IC-35, IC-36, IC-38, IC-41, IC-50 | Original scenario contract retained |
| SC-13 / P1 | Pure-fluid refrigeration loop | IC-01, IC-02, IC-21, IC-29, IC-35, IC-36, IC-38, IC-41, IC-50, IC-52, IC-54, IC-60, IC-61, IC-68, IC-69 | SE-14 |
| SC-14 / P2 | Mixed-refrigerant phase change | IC-03, IC-04, IC-06, IC-18, IC-21, IC-23, IC-25, IC-26, IC-29, IC-31, IC-35, IC-36, IC-38, IC-41 | Original scenario contract retained |
| SC-15 / P2 | Assay-derived petroleum pseudocomponents | IC-01, IC-02, IC-03, IC-10, IC-11, IC-13, IC-14, IC-16, IC-17, IC-20, IC-21, IC-24, IC-61, IC-62, IC-68, IC-69 | SE-11 |
| SC-16 / P2 | Black-oil or other reduced petroleum representation | IC-03, IC-04, IC-06, IC-07, IC-11, IC-13, IC-15, IC-24, IC-25, IC-26, IC-31, IC-34, IC-47, IC-51, IC-53, IC-57 | Original scenario contract retained |
| SC-17 / P1 | Specified-conversion reaction | IC-05, IC-07, IC-21, IC-35, IC-36, IC-41, IC-43, IC-44, IC-59, IC-71 | SE-04 |
| SC-18 / P1 | Chemical-equilibrium reaction | IC-05, IC-06, IC-07, IC-21, IC-22, IC-29, IC-35, IC-42, IC-43, IC-44, IC-45, IC-46, IC-71 | SE-03, SE-04 |
| SC-19 / P2 | Kinetically controlled reaction | IC-04, IC-25, IC-28, IC-34, IC-37, IC-38, IC-39, IC-43, IC-44, IC-45, IC-48 | Original scenario contract retained |
| SC-20 / P2 | Reactive separation | IC-21, IC-22, IC-29, IC-34, IC-35, IC-36, IC-40, IC-41, IC-43, IC-44, IC-45, IC-50, IC-52, IC-53, IC-54, IC-59, IC-60, IC-71 | SE-12 |
| SC-21 / P2 | Electrolyte mixing and neutralization | IC-05, IC-07, IC-21, IC-25, IC-35, IC-43, IC-44, IC-45, IC-46, IC-47, IC-51, IC-71 | SE-03 |
| SC-22 / P2 | Reactive gas absorption into aqueous liquid | IC-14, IC-20, IC-21, IC-24, IC-28, IC-29, IC-35, IC-37, IC-41, IC-43, IC-44, IC-45, IC-46, IC-48, IC-50, IC-59, IC-60, IC-71 | Original scenario contract retained |
| SC-23 / P2 | Inert solids carried with fluid | IC-03, IC-05, IC-06, IC-07, IC-18, IC-22, IC-25, IC-29, IC-30, IC-31, IC-38, IC-42, IC-43, IC-50, IC-71 | SE-07 |
| SC-24 / P2 | Crystallization and precipitation | IC-21, IC-22, IC-29, IC-30, IC-34, IC-35, IC-41, IC-42, IC-43, IC-44, IC-45, IC-53, IC-56, IC-59, IC-60, IC-71 | Original scenario contract retained |
| SC-25 / P2 | Gas–solid chemical transformation | IC-06, IC-21, IC-22, IC-29, IC-37, IC-41, IC-42, IC-43, IC-44, IC-45, IC-59, IC-60, IC-71 | Original scenario contract retained |
| SC-26 / P2 | Rate-based nonequilibrium contacting | IC-28, IC-29, IC-34, IC-35, IC-37, IC-38, IC-48, IC-50, IC-52, IC-54, IC-60 | SE-06, SE-12, SE-14 |
| SC-27 / P3 | Adsorption and membrane state extensions | IC-05, IC-06, IC-07, IC-09, IC-18, IC-21, IC-25, IC-26, IC-28, IC-33, IC-35, IC-37, IC-43, IC-71 | SE-09 |
| SC-28 / P1 | Heat exchange between different property packages | IC-05, IC-18, IC-21, IC-34, IC-38, IC-41, IC-48, IC-49, IC-50, IC-56, IC-59, IC-71 | SE-05, SE-13 |
| SC-29 / P2 | Material transfer across a property-package boundary | IC-10, IC-14, IC-18, IC-21, IC-35, IC-37, IC-49, IC-51, IC-54, IC-62 | SE-04, SE-05 |
| SC-30 / P2 | Translation between material representations | IC-04, IC-05, IC-07, IC-25, IC-43, IC-47, IC-51, IC-58, IC-64, IC-65, IC-69 | SE-03, SE-05 |
| SC-31 / P2 | Mass-based empirical and nonconventional materials | IC-03, IC-06, IC-18, IC-24, IC-25, IC-26, IC-28, IC-31, IC-34, IC-38, IC-41, IC-53, IC-56, IC-57, IC-71 | SE-07 |
| SC-32 / P3 | Polymer distributions and material attributes | IC-03, IC-06, IC-07, IC-08, IC-16, IC-20, IC-25, IC-26, IC-28, IC-32, IC-47, IC-51, IC-62, IC-68, IC-71 | SE-08 |
| SC-33 / P3 | Inventory-based state and dynamic compatibility | IC-25, IC-26, IC-27, IC-28, IC-35, IC-48, IC-71 | SE-10 |
| SC-34 / P3 | Restricted-equilibrium and metastable-state requests | IC-22, IC-29, IC-30, IC-35, IC-41, IC-42, IC-54, IC-56, IC-57, IC-72 | SE-15 |


### Integrated journeys and predecessor fixtures

The eight B4 integrated journeys and eight B4 synthetic fixtures are preserved verbatim in the register and unchanged in the bundled B4 files. The new SE examples supplement their information interpretation; they do not change the original fixtures' not-executed status.

| Journey | Scope | Original acceptance purpose |
| --- | --- | --- |
| IJ-01 | SC-01, SC-04, SC-05, SC-06 | Verify unit material/energy balances, reference-versus-actual compressor states, PH targets, phase allocation, and recycle convergence under one captured model revision. |
| IJ-02 | SC-07, SC-08, SC-09, SC-10, SC-11, SC-26 | Verify local-state independence, nonideal phase behavior, multiple-liquid mapping, stage/whole-unit balances, and declared contacting formulation. |
| IJ-03 | SC-02, SC-03, SC-12, SC-13, SC-14, SC-28 | Verify within-side caloric consistency, quality semantics, inverse state operations, no species exchange across the wall, and utility-loop closure. |
| IJ-04 | SC-17, SC-18, SC-19, SC-20 | Verify that each variant retains its own species authority, reaction-energy convention, required coupled conditions, and outer convergence. |
| IJ-05 | SC-21, SC-22, SC-23, SC-24, SC-26, SC-30 | Track true/apparent species, external exchanges, energy, inert versus precipitating solids, and qualified contacting formulation across all boundaries. |
| IJ-06 | SC-15, SC-16, SC-29, SC-30 | Reconstruct original assay/cuts and complete required caloric data; verify translation constraints and report irreversible aggregation/model discrepancy. |
| IJ-07 | SC-23, SC-25, SC-31 | Execute justified mass/energy operations without fictitious molecular properties; require additional explicit chemistry before the reactive extension. |
| IJ-08 | SC-27, SC-32, SC-33, SC-34 | Show meaningful representation, quantity/basis and conservation semantics, permitted actions, and qualified unavailable numerical operations. |

## 10. Step-5 open-item disposition and downstream decisions

This stage resolves information meaning and ownership. It does not select actual parameter sets, model transformations, matching thresholds, native containment strategies or numerical fixtures. Those distinctions are intentionally separated below.

| Inherited item / owner | Disposition | Concepts | Remaining qualification |
| --- | --- | --- | --- |
| PO-01 / P04 | Semantic record and ambiguity policy specified; numerical matching remains open | IC-29, IC-30, IC-58 | Within-result IDs are local; correspondence supports continuation, appearance/disappearance, split/merge and ambiguity. No universal matching score or empirical threshold is selected. |
| PO-02 / P03 | Convention and transformation semantics specified; actual profile transformations remain open | IC-21, IC-43, IC-47, IC-51, IC-71 | A reference transform may be component/state dependent. Reaction energy and chemical standards must be reconciled. Real provider/data pairs still need evidence and numerical tests. |
| PO-03 / P05 | Contribution information specified; action lifecycle and realized equations remain open | IC-35, IC-36, IC-37, IC-39, IC-40, IC-57 | Local variables, authority, residual units, scaling, derivative axes and initialization restoration are defined without a chosen equation IR or solver. |
| PO-04 / P08 | Lifecycle information specified; containment implementation untested | IC-53, IC-54, IC-55, IC-70 | Health/isolation/recovery and cancellation authority are explicit. A tested reentrant/serialized/process-isolated strategy must still be selected per build. |
| PO-05 / P10 | Dependency/currency semantics specified; selective invalidation algorithm open | IC-64, IC-65, IC-66, IC-67 | Capture exact relevant dependencies or a conservative superset. Guarded publication and conservative invalidation are specified; no incremental engine chosen. |
| PO-06 / P02 | Information needs specified; numerical petroleum/empirical coverage still open | IC-01, IC-03, IC-14, IC-16, IC-17, IC-20, IC-24 | Raw assay, cut definitions, method/data/caloric completion and limited mass-only descriptions are explicit. Actual complete routes are not inferred. |
| PO-07 / P03 | Information needs specified; chemistry/transport combinations still open | IC-21, IC-38, IC-40, IC-44, IC-45, IC-50 | Phase/domain/pair, diffusion frames, caloric/chemical standards and bulk/interface context are identified; real SC-20–26 profile qualification remains required. |
| PO-08 / P09 | Check information and independence categories specified; fixture thresholds open | IC-41, IC-59, IC-60, IC-61 | Original targets, scales, required/optional status, unknown evidence and independent references have fields. No universal physical accuracy threshold is introduced. |
| PO-09 / P08 | Manifest and opacity semantics specified; distribution facts remain open | IC-10, IC-14, IC-20, IC-53, IC-58, IC-62, IC-68 | Exact bundle/build/binding/data attribution with explicit undisclosed choices is admitted. Public-availability/license judgments are inherited research limits, not refreshed claims. |
| PO-10 / P01 | Extension information specified; action review and R/E/V remain pending | IC-08, IC-09, IC-25, IC-32, IC-33, IC-35, IC-42, IC-71 | Concrete support/weighting, denominator, inventory and restriction meanings now exist. Step 7 must complete actions; no numerical P3 commitment is added. |

### What Step 7 should now consume

The action blueprint can refer directly to the dictionary instead of inventing new payload meaning per operation. Authoring and preparation actions consume IC-01–24 and create validated proposals. Local property/state/chemistry actions capture IC-25–47 and the effective location IC-48–52. Execution uses IC-53–58/72; acceptance uses IC-59–63; change, archive and publication use IC-64–70. Actual transfer quantities are IC-71 wherever they occur.

For each action, specify exactly which revisions/roles it consumes, which candidate/proposal it can create, what may be mutated only privately, which checks must complete, and which event is the guarded adoption point. Required paths include successful, incomplete, blocked, failed-check, failed-provider, cancelled, superseded and obsolete-context outcomes. Numerical recovery must preserve or explicitly replace the physical problem. No action may introduce a new untyped interpretation of material, phase, parameter or result.

### Library mapping remains inherited, not reselected

The dictionary supports a complete DWSIM/ChEDL/Clapeyron configuration, a focused ThermoPack/CoolProp/FeOS realization, a coherent Reaktoro chemical system, and an IDAES-style mathematical contribution where the actual interface is qualified. CAPE-OPEN continues to inform interoperability semantics. These are the B3/B5 candidate roles, not newly tested capability claims. Every adapter must translate its own identities, bases, phase slots, conventions and lifecycle facts into this semantic boundary without inheriting blanket upstream coverage. Whole-provider reuse remains valid when it preserves coherence better than extracting primitives.

## 11. Audit, evidence limits and completion gate

| Structural check | Result |
| --- | --- |
| 72 unique concepts | PASS |
| concept fields complete | PASS |
| unique fields per concept | PASS |
| cardinalities valid | PASS |
| one valid semantic owner per concept | PASS |
| field concept references valid | PASS |
| relationship endpoints cardinalities valid | PASS |
| unique relationship ids | PASS |
| 94 requirement ids preserved | PASS |
| primary concept owner matches step5 | PASS |
| all concepts have requirement trace | PASS |
| requirement classes preserved | PASS |
| original requirement test links preserved | PASS |
| 18 information products preserved | PASS |
| product owners preserved | PASS |
| product primary concept owner matches | PASS |
| product decomposition owners match | PASS |
| every concept in exactly one product decomposition | PASS |
| 34 scenarios and profiles preserved | PASS |
| scenario information routes present | PASS |
| all rules have valid concepts and witnesses | PASS |
| all examples have valid concepts and scenarios | PASS |
| all P3 scenarios have concrete examples | PASS |
| step5 open item owners preserved | PASS |
| source files present | PASS |
| step5 recorded source hashes match | PASS |
| step4 recorded source hashes match | PASS |
| authored arithmetic and guard logic checks pass | PASS |
| all concepts have positive negative examples | PASS |
| unchanged step5 contract graph acyclic | PASS |


### Counts

| Item | Count |
| --- | --- |
| concepts | 72 |
| semantic_fields | 372 |
| relationship_contracts | 108 |
| cross_concept_rules | 28 |
| worked_semantic_examples | 15 |
| functional_requirements | 94 |
| information_products | 18 |
| scenarios | 34 |
| primary_packages | 10 |
| structural_checks | 30 |
| authored_arithmetic_checks | 22 |


### Authored example checks

| Check | Obtained | Expected | Status |
| --- | --- | --- | --- |
| PH target formula | 50 | 50 | PASS |
| False-success enthalpy residual | 30 | 30 | PASS |
| Two-liquid A total | 5 | 5 | PASS |
| Two-liquid B total | 5 | 5 | PASS |
| Binary reference shift z=(1/2,1/2) | 25 | 25 | PASS |
| Binary reference shift z=(3/4,1/4) | 35/2 | 35/2 | PASS |
| Reaction reference delta at extent 0.4 | 12 | 12 | PASS |
| Residual model discrepancy after stated offset | 10 | 10 | PASS |
| Temperature for corrected H=50 | 1025/3 | 1025/3 | PASS |
| Frozen versus response slope difference | 1 | 1 | PASS |
| Constrained composition derivative x1 | -3 | -3 | PASS |
| Constrained composition derivative x2 | -2 | -2 | PASS |
| Empirical heating duty | 100 | 100 | PASS |
| Mass-weighted histogram first bin | 7/20 | 7/20 | PASS |
| Mass-weighted histogram second bin | 13/20 | 13/20 | PASS |
| Surface amount at stated loading | 1 | 1 | PASS |
| Loading after transfer | 11/50 | 11/50 | PASS |
| Bulk plus surface conservation | 3 | 3 | PASS |
| Illustrative publication conjunction True/True/True | True | True | PASS |
| Illustrative publication conjunction False/True/True | False | False | PASS |
| Illustrative publication conjunction True/False/True | False | False | PASS |
| Illustrative publication conjunction True/True/False | False | False | PASS |

### Interpretation

The audit checks concept uniqueness, semantic content, cardinality syntax, reference endpoints, ownership consistency, predecessor hashes, every requirement/scenario route, and exact authored arithmetic. The unchanged Step-5 contract-dependency graph was rechecked; the semantic-reference graph is deliberately not required to be acyclic.

This is not a proof of complete semantic sufficiency for every future physical theory, nor a provider conformance, performance, fault-recovery, concurrency or empirical thermodynamic test. No original AT, PV, P or scenario R/E/V status was upgraded. The new examples exercise algebra and relationship meanings only. Real-fluid parameter data, acceptance thresholds and exact build behavior remain profile-specific work.

**Step-6 completion:** each of the eighteen information products has concrete concept decomposition and a retained owner; each concept has identity, required/conditional content, quantity/basis semantics, allowed variants, invariants, lifecycle, relationships and valid/invalid examples; every functional requirement has an information home. Steps 7–9 must turn this into complete actions, qualify actual provider combinations and execute acceptance evidence before implementation or scientific capability is claimed.

## 12. Source manifest and citation convention

B4 requirement identifiers link to the unchanged functional specification included in the bundle. B5 P/A/PD/PO identifiers refer to the unchanged conceptual packaging baseline. B2/B3 contain the original pinned upstream research and its limitations. No new external library assertion is established by this document.

| Key | File | SHA-256 |
| --- | --- | --- |
| B1 | [thermodynamics_simulation_behavior_scope_v0_1.md](thermodynamics_simulation_behavior_scope_v0_1.md) | `1d12284bbca6569f198caaae86327788bac8fbaf3946625d99d971ef3602d028` |
| B2 | [dwsim_workflow_reverse_engineering_v0_1.md](dwsim_workflow_reverse_engineering_v0_1.md) | `55e77b6fddab4547280e4e40ca5c742fac3fd322e45534767ddae7f5f6ba94ab` |
| B3 | [thermodynamic_package_comparative_research_v0_1.md](thermodynamic_package_comparative_research_v0_1.md) | `c0f9216c450405360b8532771e6c4beca6536154345ea5adf00130a9c4e73875` |
| B4 | [thermodynamics_functional_requirements_v0_1.md](thermodynamics_functional_requirements_v0_1.md) | `7b9c3c4e2110531cb5767fb00f9ba11dec160294b8cafd909c5b04984a87b5c2` |
| B4-register | [thermodynamics_functional_requirements_v0_1_register.json](thermodynamics_functional_requirements_v0_1_register.json) | `1777f2c60962526252aa863dc2cb673f218b4ed58fe9c108f9721264f9fde825` |
| B5 | [thermodynamics_conceptual_packaging_v0_1.md](thermodynamics_conceptual_packaging_v0_1.md) | `85602e0a750add6a1168cb4f4294998feb5e90e35b37cef5f0b4e63c5f1076f1` |
| B5-register | [thermodynamics_conceptual_packaging_v0_1_register.json](thermodynamics_conceptual_packaging_v0_1_register.json) | `d46ce687dee20cb7e677082039f349a38d8fd71101a659079d47bb8d57d80612` |

All predecessor bytes are included unchanged. The JSON companion is a semantic-catalog management register, not a proposed runtime/storage schema. Physical JSON/Arrow/database layouts, units libraries, expression engines, solver APIs and Rust boundaries are intentionally not selected here.

**Design thesis:** every number must retain what it measures, which material and basis it describes, who supplied it, what was allowed to change, which definitions produced it, what has been checked, and whether it is current. The information structures make those distinctions explicit without forcing every thermodynamic engine to share the same internals.
