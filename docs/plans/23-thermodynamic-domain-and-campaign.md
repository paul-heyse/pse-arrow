---
title: Thermodynamic domain model and integrated kernel campaign
status: in-progress
date: 2026-09-29
adrs: [ADR-0123, ADR-0124, ADR-0125, ADR-0127]
review_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md]
scenario_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s01]
---

# Thermodynamic domain model and integrated kernel campaign

**Status: authorized 2026-09-29; P0 in progress.** This plan owns the typed domain model,
the Plan 21 K9 campaign (transferred here, re-scoped) and the open Plan 20 finding
dispositions F01–F13.

## Context

Plan 21 built the modeling kernel (K0–K8, 2026-09-27) and left its K9 integrated
campaign proposed. Before a wide array of thermodynamic libraries is ported, the
maintainer requires a **true domain model**: typed and relational, the single source of
truth for how domain entities, attributes and methods are expressed and actioned, with no
strings or digits carrying meaning and no library receiving implicit, ad hoc behaviour.

The current representation falls short of that in both the kernel and the knowledge:

- The modeling IR (`authored.modeling_declarations` v6) stores types, data cells,
  missing-value policies, citations and imports as UTF-8 text that admission re-parses
  (`pse-modeling/src/types.rs`, `data.rs`). Composite unit literals resolve by exact
  symbol string, so `{J/(K*mol)}` and `{J/(mol*K)}` differ.
- Entity kinds are empty names. Correlation coefficients are `Scalar` with a hidden
  `scale` column and conventions written as digits (`T/(1000 K)`, `6.02214076e-7`);
  units hide in column names (`sigma_angstrom`). Method selection lives in entity names
  (`benzene_gas_rpp4_oracle`) and column names (`liquid_fit`). Phases are declared twice
  and branched on by identity (`if p==equilibrium.liquid`, `if j==chem.water`).
  Provenance is free text (`source "…"`, `sources.md`), and quantity names are repeated
  `[[quantity_aliases]]` in every manifest.
- K9 readiness: only S11 is covered; S01 and the measurements are missing; the K8
  "77/77" seed evidence combined a retained run with two scratch selectors and was never
  one reproducible run.

## Decisions

| ID | Decision | Status | Route |
|---|---|---|---|
| T1 | The domain schema is a **typed package schema**. The kernel gains generic typed-relational mechanisms; one domain declaration in packages is the authority; every library package conforms to it. The kernel defines no scientific concept and the Plan 21 knowledge-boundary test (N1) holds. Only vocabulary the kernel acts on becomes registry enums (ADR-0115) | Maintainer decision, 2026-09-29 | ADR-0123, ADR-0127 (supersedes ADR-0126) |
| T2 | Readiness scope: data banks (NIST, RPP3/4/5, Perry, the DIPPR public subset, ChemSep, CIAAW) and method families (cubic and SAFT EoS, NRTL/Wilson/UNIQUAC/UNIFAC, eNRTL, multiparameter Helmholtz, transport, reactions). External libraries as oracles or production providers are outside it; FeOS, teqp and IDAES values remain frozen oracle data | Maintainer decision, 2026-09-29 | This plan |
| T3 | Unit literals are canonical unit products; coefficient types are declared derived quantity kinds, resolved, never synthesized | Proposed | ADR-0124 |
| T4 | Large datasets are Parquet package data documents admitted through the typed relation path; inline cells remain for seed-size data. This decides Plan 21's storage open item | Proposed | ADR-0125 |
| T5 | K9 transfers here. Review scenarios are cited as `CT-S01…CT-S14` (Plan 22's companion defines other S10–S25); new domain scenarios are `DM1–DM6`; K9 measurements R1–R3 become `M1–M3` | Adopted | This plan |
| T6 | The knowledge-boundary audit is a reasoned review, not a new lint or validator | Adopted | AUD |

## Architectural drivers and scenarios

### Generic kernel mechanisms

| Mechanism | Replaces |
|---|---|
| **Structured IR** (the relation version after H1's). Types are a post-order arena; data cells are typed tagged values (quantity as magnitude and unit product, reference, identifier, text, integer, boolean, missing; optional uncertainty). Missing-value policy, annotation kind, analysis-fact namespace, import version operator, key symmetry and data facets are registry enums. Names survive only as path segments that the checker resolves once to IDs | Utf8 `type_name`, cell text, policy words, annotation string dispatch, `analysis.`/`stage.` prefixes, string version comparison |
| **Unit algebra.** Canonical products of atomic units with rational exponents; defined units derive dimension and scale | Exact-symbol lookup; composite units registered whole |
| **Derived quantity kinds.** Kinds defined by a monomial (`MolarCp/Temperature^2`) with a declared canonical unit; multiplicative chains over eligible leaves (differences, dimensionless, true-zero ratio points) resolve against declared kinds; result scale, reference and basis come from the derived kind; unit products have spelling-independent identities | `Scalar` coefficients, hidden `scale`, digits in bodies |
| **Entities as typed records.** Attribute schemas, single refinement (`kind ion extends species`), opaque identifier schemes unique within the package closure. **Keyed kinds** populated by datasets, with identity framed over the key-declaring kind and its typed key values (uniqueness across refinements; enum members have identities); **kind-level bindings** of inherited attributes, including functions; a static `Ref` resolves the most-derived kind's function and specializes per resolved function (non-static `Ref` refused); typed `constant` declarations | Empty kinds, name-encoded fit entities, zero-argument constant functions |
| **Relations with constraints.** Primary keys, `Ref`/`Row` foreign keys, `complete_over`, pair symmetry with a diagonal policy, uniqueness, integer-range keys, derived columns and row `require` clauses, checked at admission in three phases: key identities; order-free reference, completeness and symmetry checks; value-dependency-ordered derived columns and requirements (row-level cycles refused) | Fixed-point rescans; lookup-time "required value absent"; every ordered kᵢⱼ row written out |
| **Typed provenance.** Datasets, constants and tests name a source entity (a kind with the provenance facet) and a role. Roles are package enumeration members that declare kernel facets (`test_only`, `requires_lineage`). Test-only taint propagates along resolved row references and is the one authority for "production reads no test-only data" | `source`/`revision` text; `sources.md` prose |
| **Typed envelopes.** Form (function `valid`), data (declared envelopes) and closure (`annotation valid`) layers intersect; the property package or analysis selects the extrapolation policy per layer; a form declares which arguments or integration interval each envelope guards; checks name their layer | The `minimum`/`maximum` column-name convention |
| **Naming once.** Quantity types and reference states are named in the physical document; reference states are addressable records with typed T and p; imports resolve by package identity and typed version requirement | Repeated `[[quantity_aliases]]`; datum functions; name-based imports |
| **Package data documents.** Package documents become bytes with a declared kind; `data/*.parquet` is decoded by Arrow type in `pse-runtime` into an Arrow-free row set and admitted through the same relation path; units come only from the declaration; content hashes enter the package checksum and source revision; admitted tables are Salsa-tracked queries | No bulk route; per-cell expression evaluation |

### Thermodynamic domain schema (package knowledge)

- **Chemical core in a `chemistry` module of `pse.physical`** (ADR-0127). It holds the
  species, element, reaction and phase kinds with their attribute schemas, the identifier
  schemes, the formula relation and the single phase authority, typed by
  `compatibility.PhaseType`. The kinds belong here because 177 physical quantity types name
  species, element or reaction as their subject, six operations name them as result
  subjects, and the finite reductions range over species. The physical inventory is admitted
  from `pse.physical` alone; register R-51 defers composing it across packages. The 354
  shaped quantity types over chemistry axes are unreferenced and are deleted in SM0.
- **`packages/reference/domain` (`pse.domain`)**, above `physical`, holds the rest of the
  schema:
  - provenance kinds and roles;
  - property kinds;
  - the abstract keyed parameter set;
  - pair and group relations;
  - `selection`, `pair_selection` and `component_role`;
  - constants.
- **`methods`** becomes the form library: each form is a refined parameter-set kind with
  dimensioned coefficients and bound functions.
- **`data/<bank>`** packages carry rows only: the species catalogue, `ciaaw`, `nist`,
  `perry7`, `poling2000`, the Parquet bank `gross-sadowski-2001` and oracle banks
  `oracles/{idaes-2.13, teqp-0.23.1, feos-0.10.1}`.
- **`thermodynamics`** reads properties only through `selection`; **`seed-data`** shrinks
  to property-package bindings and fixtures; a new **`campaign`** package holds the K9
  flowsheets and studies.

```pse
// pse.physical: chemistry module
identifier scheme cas; identifier scheme inchikey;               // opaque; uniqueness only
entity kind element { key symbol: Text; atomic_number: Count unique; standard_atomic_weight: MolarMass; }
entity kind species { cas: Id<cas>? unique; inchikey: Id<inchikey>? unique; charge: ChargeNumber = 0;
  derived molar_mass: MolarMass = sum(e in element | formula[self,e]*e.standard_atomic_weight); }
relation formula[j: Ref<species>, e: Ref<element>]: Count missing zero;
entity kind apparent extends species {}
relation dissociation[a: Ref<apparent>, j: Ref<species>]: Count
  require each a: sum(j | dissociation[a,j]*j.charge) == a.charge;
entity kind phase { type: compatibility.PhaseType; }
// pse.domain: provenance, properties and selection
enum ProvenanceRole { published facets(); measured facets(); derived facets(requires_lineage);
                      fitted facets(requires_lineage); synthetic facets(test_only);
                      oracle_input facets(test_only); }
entity kind source { title: Text; }
entity kind publication extends source { year: Count; locator: Text?; doi: Id<doi>?; }
entity kind software_release extends source { version: Text; }
entity kind oracle_test extends source { release: Ref<software_release>; locator: Text; }
entity kind property { quantity: QuantityType; shape: IndexShape; applies: Set<compatibility.PhaseType>; }
entity kind parameter_set {                                         // abstract, keyed
  key subject: Ref<species>; key property: Ref<property>; key phase_type: compatibility.PhaseType;
  key source: Ref<source>; key variant: Count = 1;           // datum comes from the form's result types
  require phase_type in property.applies; }
entity kind caloric_set extends parameter_set {
  cp: Fn(T: Temperature, s: Ref<caloric_set>) -> MolarCp;
  dh: Fn(T0: Temperature, T: Temperature, s: Ref<caloric_set>) -> DeltaH;
  ds: Fn(T0: Temperature, T: Temperature, s: Ref<caloric_set>) -> DeltaS;
  envelope T: Temperature guards(cp.T, dh.[T0,T], ds.[T0,T]); }
relation pair[s: Ref<source>, p: Ref<property>, i: Ref<species>, j: Ref<species>]: Scalar
  symmetric(i, j) missing require;
entity kind main_group {}  entity kind subgroup { main: Ref<main_group>; r: Scalar; q: Scalar; }
relation group_count[j: Ref<species>, g: Ref<subgroup>]: Count missing zero;
entity kind property_package {}
relation selection[k: Ref<property_package>, j: Ref<species>, p: Ref<property>,
                   t: compatibility.PhaseType]: Ref<parameter_set>
  require value.subject == j and value.property == p and value.phase_type == t;
relation pair_selection[k: Ref<property_package>, p: Ref<property>]: Ref<source>;
relation component_role[k: Ref<property_package>, j: Ref<species>]: compatibility.ComponentType
  require (value == Cation implies j.charge > 0) and (value == Anion implies j.charge < 0);
constant gas_constant: GasConstant = 8.314462618{J/(mol*K)} provenance(si_2019, published);
// methods: a form is a refined kind; data banks add rows only
entity kind dippr100 extends caloric_set {
  c1: MolarCp; c2: MolarCp/Temperature; c3: MolarCp/Temperature^2; c4: ...; c5: ...;
  cp = dippr100_cp; dh = dippr100_dh; ds = dippr100_ds; }
dataset perry7_liquid_cp: methods.dippr100 provenance(perry7_t2_196, published) {
  [benzene, cp, liquidPhase] =
    [129440{J/(kmol*K)}, -169.5{J/(kmol*K^2)}, 0.64781{J/(kmol*K^3)}, 0, 0; T in 278.68{K}..353.24{K}]; }
entity kind cubic_family { omega_a: Scalar; omega_b: Scalar; u: Scalar; w: Scalar; }
relation kappa[f: Ref<cubic_family>, k: 0..2]: Scalar;
```

The sketch is the target shape; exact syntax is settled by KR3–KR7. The property package
binds its phase formulation (equation of state or gᴱ family) as an ordinary definition
binding. A generic
`fn cp(k, p: Ref<phase>, j, T) = s.cp(T, s) where s = selection[k, j, thermo.cp, p.type]`
dispatches on the concrete form. A missing selection row is a typed refusal naming the
package, species, property and phase type. Two-phase formulations take phase slots
(`param liquid: Ref<phase> require liquid.type == liquidPhase`); the solvent comes from
`component_role`.

### Today's carriers and their replacements

| Today | Becomes | Packet |
|---|---|---|
| `caloric.fit` and 13 name-encoded fits | Keyed `parameter_set` rows | SM3 |
| 354 unreferenced physical shaped types over chemistry axes; chemistry kinds in `kinds` | Deleted; chemistry kinds in the `chemistry` module of `pse.physical` | SM0 |
| `polynomial` with `scale` and `Scalar a..e`; Shomate `t=T/(1000 K)` | Refined forms with dimensioned coefficients; any reduction scale is a typed attribute | KR2, SM3 |
| `caloric_binding[j]{liquid_fit,…}`, `vapor_pressure_fit[j]` | `selection` rows of a `property_package` | SM5 |
| `if p==equilibrium.liquid`, `if j==chem.water`, duplicated phases | Phase slots with `require`; `component_role`; one phase authority | SM0, SM5 |
| `sigma_angstrom:Scalar`, `6.02214076e-7`, PR Ω digits, `gas_constant_value()`, p0..p6 with `power` | `sigma: Length`; typed constants with provenance; `cubic_family`/`kappa`; integer-range keys | SM4 |
| `source "…"`, test `source`/`revision`, `sources.md` prose, `crates/pse-kernels/data/*.json` | Source entities and roles; oracle Parquet banks | SM2 |
| `atoms`/`atomic_mass` free tables; molar mass only in a test | `formula`, `element.standard_atomic_weight`, derived `species.molar_mass` | SM1 |
| 42 `[[quantity_aliases]]`; oracle enthalpy aliases; `enthalpy_datum()` | Named quantity types and reference states in the physical document | KR8, SM4 |
| Registry `reference.constants` (empty) | Deleted; `constant` declarations | SM4 |

### Scenarios

Original K9 scenarios keep their review definitions
([S01](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s01)–S14);
the new domain scenarios are defined here.

| ID | Scenario | Acceptance |
|---|---|---|
| DM1 | Add a data bank from Parquet | ≥20 non-associating PC-SAFT species keyed by CAS admit with zero Rust; published values and FeOS agreement for two new species |
| DM2 | One species in two banks | Both banks admit; the package selects one (pure components through `selection`, pair data through `pair_selection`); duplicate CAS entity, same-key conflict and unknown CAS in a Parquet row are refused |
| DM3 | Wrong coefficient units | Refusal names the attribute and the expected and actual quantity |
| DM4 | Oracle data in a production package | A production root reading a test-only (`oracle_input`) row is refused by the kernel taint; the same bank row read by a fixture passes |
| DM5 | Add a method family as data | NRTL gᴱ with ln γ from defaults: closed form, Gibbs–Duhem and IDAES oracle agree |
| DM6 | Publish fitted parameters | A `fitted` set carries lineage to its fit; one without lineage is refused |

## Plan

Each packet compiles (`just check-package`/`just check`), runs its named targeted tests,
runs `just codegen` after registry edits (and `just python-stubs` where Python changes),
and deletes what it replaces in the same change. No fixture is removed unless a
replacement covers all of its assertions; package-data packets report H1 counts before
and after.

### W0 — coordinator

| Packet | Responsibility / dependencies | Acceptance | Deletion | Status |
|---|---|---|---|---|
| P0 Decisions and plan transfer | ADR-0123–0126 accepted after review; Plan 21 Outcome, K9 and F01–F13 transfer, retirement of Plan 21 records; register rows; current-work indexes | `just adr-lint` | Plan 21, its execution packets, the K0–K3/K4–K7/through-K8 reviews (ADR-0096) | done (`0e725de2`, `41336c23`) |
| H1 Whole-seed conformance | Typed fixture `policy { backend; presolve; derivatives; expansion; body slots; time limit }` (within ADR-0119); `packages/reference/conformance.toml`; `just seed-conformance` | `kernel_conformance_reads_fixture_policies_from_declarations`, `kernel_conformance_refuses_unknown_fixture_policy_setting`, `test_conformance_runs_the_declared_reference_set`; one complete run of every fixture | `--fixture-solver/--fixture-presolve/--fixture-derivatives` flags, the runtime `ModelingFixturePolicy` map and parameter, `WorkflowError::FixtureIntentConflict` (the declaration-ID fixture subset in `tests/support/plan14.rs` stays: it selects fixtures for targeted Rust tests) | done; first complete run 90 passed, 1 failed, 3 inconclusive of 94 (H1f) |
| H1f Whole-seed findings; H1 | The first complete run's non-passing fixtures, each fixed at its cause: SCIP's native allowance is unreachable from a fixture (`heater_optimization_certified` fails; register R-40); derivative sampling perturbs an input fixed at its bound (`saturated_integrated` inconclusive); the tangent-plane check miscounts compared entries (`tpd_ideal` inconclusive); automatic selection now stops `pfr_radau` at a local infeasibility where K8 passed (a routing or Ipopt regression, not a fixture policy) | `just seed-conformance` 94/94 with no masking policy | Whatever each root cause replaces | done: `just seed-conformance` 94/94 (5 min 18 s, 128 GiB pool, memory-capped). SCIP's allowance is declared per solve (`limits foreign_bytes`, `SolveControls.foreign_bytes`; R-40 removed). The derivative sample steps one-sided at active bounds and does not expect comparisons along pinned coordinates. `pfr_radau` had passed K8 only by luck of MUMPS ordering; the fix is two presolve defects: affine rows now propagate from their proof, and automatic presolve declines only the offending propagation. `authored.modeling_declarations` is now version 8 |
| SM0 Chemistry kinds and phase authority | Confirm the modeling path needs no registry-shaped chemistry types and delete the 354 unreferenced shaped types from `physical.yaml`; move the species, element, reaction and phase kinds (identities unchanged) and the canonical phases into a `chemistry` module of `pse.physical`; references move from `kinds.*` and `equilibrium.liquid/vapor` | `package_declared_axis_indexes_a_sum_without_a_registered_shaped_type`; H1 unchanged | 354 shaped quantity types, `kinds.phase_species`, the four kinds in `kinds.pse`, `equilibrium.liquid/vapor`, `chem.liquid/vapor/aqueous` | done (on main via `plan23/int`, `5664e80a`) |

### Track K — kernel (sequential; owns `pse-schema`, the parser and `pse-modeling/data.rs`)

| Packet | Responsibility / dependencies | Acceptance (targeted tests) | Deletion | Status |
|---|---|---|---|---|
| KR1 Unit algebra | `dsl` `Number.unit` → `UnitProduct`; `QuantityRegistry::compose`; atomic and defined units; affine only as sole factor; unit-product identity framed over canonical factors; `reference.units` v2 | `unit_product_is_order_independent` (identity equality), `composite_literal_needs_no_registered_whole_unit`, `affine_unit_only_as_sole_factor`, `defined_unit_dimension_and_scale_are_derived`, `rational_unit_exponents_canonicalize`, `report_in_a_composite_unit_carries_its_identity` | Token-joining `unit()`, exact-string `unit_by_symbol` call sites, authored scale of composite units | done (on main, `5664e80a`) |
| KR2 Derived quantity kinds; KR1 | `QuantityKind.definition` with canonical unit, `by_monomial` index, chain flattening in `infer.rs` with the ADR-0124 leaf eligibility and result policy; language type expressions use the same lookup | `seed_forms_type_without_intermediate_kinds` (dippr100, Shomate, RPP4 `cp`, `dh`, `ds`), `eligible_leaves_are_exactly_true_zero_ratio_points_and_differences`, `nonzero_datum_leaf_is_refused_with_its_factor`, `basis_must_agree_or_be_declared`, `undeclared_monomial_is_refused_with_factors`, `rule_and_chain_disagreement_is_refused`, `type_expression_resolves_by_monomial` | — | on `plan23/kr` (`a8a3282a`); awaiting merge |
| KR3 Structured IR core; KR2 | Type arena replaces every type-bearing Utf8 (relation version after H1's); `ModelingMissingPolicy`, `ModelingAnnotationKind`, fact-namespace enum, `ModelingVersionOperator`; frame `ModelingSourceRevisionV2` with the ADR-0123 Outcome 8 preimage, and new variants for every frame whose preimage changes (dispatch-body and function-specialization frames included), with golden vectors; no migration (§20.5) |  `type_arena_render_parse_roundtrip` (property test), `type_arena_rejects_forward_child`, `missing_policy_is_enum`, `annotation_kind_dispatch_is_exhaustive`, `analysis_facts_are_typed_namespaces`, `import_requirement_is_typed`, `source_revision_changes_with_a_physical_name_binding`, `unchanged_inputs_reproduce_the_source_revision` (the data-byte case lands with KR9: `source_revision_changes_with_one_data_byte`) | String type grammar in `types.rs`, policy-word check, annotation string dispatch, `analysis.rs` prefix parsing, `Frame::ModelingSourceRevisionV1` | done (on main; version 9) |
| KR4 Typed cells and entity records; KR3 | `ModelingCell`; attribute schemas; kind refinement (interface graph and SCC check reused); identifier schemes and closure uniqueness; keyed kinds from datasets (identity over the key-declaring kind and typed keys; enum members gain identities; dataset-supplied keys are declared bindings); kind-level bindings incl. functions; static `Ref` calls specialize per resolved most-derived function on the existing function-specialization path; `constant` declarations | `kind_extends_kind_and_subkind_members_conform`, `kind_refinement_cycle_refused`, `identifier_unique_within_closure`, `identifier_values_are_never_interpreted`, `attribute_cells_type_checked_at_admission`, `missing_required_attribute_refused_at_admission`, `keyed_identity_is_the_key_declaring_kind_and_typed_keys`, `same_key_in_two_forms_is_refused`, `moving_a_row_between_forms_keeps_its_identity`, `refined_kind_binds_inherited_function_attribute`, `ref_calls_dispatch_one_body_per_concrete_kind`, `non_static_ref_is_refused`, `constants_are_typed_declarations` | Per-access attribute re-evaluation; interface-only base rule for kinds | done (on main; `authored.modeling_declarations` version 10; frames `ModelingKeyedEntityV1`, `ModelingCoordinateV2`, `ModelingFiniteFunctionV3`; enumeration members carry identities) |
| KR5 Relations with constraints; KR4 | `data.rs` rewrite: three-phase admission (key identities; order-free references, completeness, symmetry, uniqueness, integer ranges; value-dependency-ordered derived columns and row `require`), direct typed ingestion, positional rows | `row_reference_requires_existing_target_row`, `self_referential_lineage_admits_and_cycles_are_refused`, `completeness_over_declared_sets_checked_at_admission`, `required_without_completeness_refused`, `symmetric_pair_answers_both_orientations`, `symmetric_pair_with_both_orientations_refused`, `unique_constraint_rejects_duplicate_tuple`, `derived_column_evaluated_once_per_row`, `row_requirement_names_row_and_clause`, `lookup_outside_completeness_set_refused_before_evaluation` | Fixed-point loop, per-cell text evaluation, string missing-policy test, lookup-time absence failure | done (on main; `authored.modeling_declarations` version 11). Library tables declare open keys (`complete_over(j)`); each dataset claims the set its rows cover (`complete_over(j in S)`), ratified. The Python contract generator now emits named structures in dependency order |
| KR6 Typed provenance and facets; KR5 | Dataset/constant/test `source` and role; roles as package enum members declaring registry facets (`test_only`, `requires_lineage`); row-origin role provided by the kernel; test-only taint along resolved row references (static where keys are literal, else at specialization); `runtime.modeling_conformance` v2 `oracle_source_id` | `dataset_source_requires_provenance_kind`, `production_root_reading_an_oracle_row_is_refused`, `shared_relation_with_oracle_rows_keeps_production_roots_admissible`, `lineage_facet_requires_acyclic_lineage`, `new_role_is_a_package_only_edit`, `conformance_publishes_oracle_source_id` | Oracle text rules, non-empty source check, verbatim oracle copy | done (on main; `authored.modeling_declarations` version 12; `runtime.modeling_conformance` version 2; refusal rule `modeling.provenance`) |
| KR7 Typed envelopes; KR5 | Declared envelopes generate guards through the domain-predicate path; forms declare the arguments or integration interval each envelope guards; the property package or analysis selects extrapolation per layer; `runtime.modeling_checks` v3 `layer` | `relation_envelope_generates_function_guard`, `envelope_bounds_must_match_axis_type`, `increment_guards_its_integration_interval`, `consumer_selects_extrapolation_per_layer`, `layers_intersect_and_form_never_extrapolates`, `validity_checks_name_their_layer` | — | done (on main; `authored.modeling_declarations` version 13; `runtime.modeling_checks` version 3 with `layer`). The seed declares envelopes on `caloric.shomate`, `caloric.polynomial`, `pure_properties.liquid_density`, `pure_properties.vapor_pressure` and `saponification.liquid_data`; the eight authored bound predicates are deleted. Lineage taints: `bt_ideal.pressure_bindings` is test-only |
| KR8 Naming once, typed imports; KR3 (second kernel track) | `reference.quantity_types` v2 `name`; named reference states with typed T and p; generic physical-reference language types (`QuantityType`, `ReferenceState`); `authored.packages` v2 typed `version_req`; imports by `package_id` | `quantity_names_declared_once_in_physical_document`, `quantity_name_requires_manifest_dependency`, `ambiguous_quantity_name_refused`, `reference_state_attributes_are_typed`, `import_requires_dependency_by_identity` | `authored.package_quantity_aliases`, `document_quantity_aliases`, every `[[quantity_aliases]]`, name threading | done (on main) |
| KR9 Package data documents; KR5, KR6 | Package documents become bytes with a declared kind (loaders, `package_checksum`, operational source documents with a binary content column after `just --yes db-reset`, worker bundle verification); `DocumentKind::Data`; `pse-runtime` Parquet decoding by Arrow type into a row set, units only from the declaration; identifier-scheme references; admitted tables as Salsa-tracked queries; rows charged to workspace limits | `parquet_dataset_admits_through_declared_relation`, `parquet_column_type_mismatch_refused`, `parquet_metadata_unit_disagreement_is_refused`, `parquet_reference_by_identifier_scheme_resolves`, `data_bytes_enter_package_checksum_and_source_revision`, `unchanged_dataset_reuses_admitted_table` (incremental equals clean), `durable_job_round_trips_a_package_with_a_data_document`, `large_table_admits_without_cell_evaluation`; 10⁵-row admission bench measured in Q | — | pending |

### Track H — harness and kernel gaps

| Packet | Responsibility / dependencies | Acceptance | Deletion | Status |
|---|---|---|---|---|
| H2 Formal-slot gap | `MAX_FORMAL_SYMBOLS` becomes a lazily extended pool up to the explicit `body_slots` limit with a typed refusal; M1 bench `modeling_preparation` (PC-SAFT 2/5/10/20 components; PR inline and nested) | `formal_pool_extends_to_the_declared_limit_and_refuses_beyond`, `formal_symbols_are_stable_across_pool_extension` | The fixed ceiling | done (on main, `5664e80a`) |
| H3 Reuse counters (CT-S08) | `PreparationCounts` in study results, Rust and Python | `study_results_carry_preparation_counts`, `test_flash_sweep_prepares_structure_once` | — | done (on main, `5664e80a`) |
| H4 Regime-crossing counters (M3) | Nested stage reports regime changes per outer iteration | `nested_stage_reports_regime_crossings_per_outer_iteration` | — | done (on main, `5664e80a`; crossings are a typed recoverable `ProviderError::RegimeCrossing`) |
| H7 Benchmark defects found by H2; H2 | Existing bench failures, all present before Plan 23: `native_cache` reserves reader leases per planned output partition, not per actual Parquet reader partition, and at four partitions exceeds its 8 MiB aggregate limit; `native_consolidation` truncates its 1 MiB plan-text capture when DataFusion folds a 65,536-row `unnest(range)` into a literal, and its 4 workers × 4 queries case is refused by CPU admission, which rejects rather than queues. The smoke-matrix selection bug is already fixed (`1dd8c350`) | `just bench-smoke`, `just bench-cache` and `just bench-production` pass, with no limit raised to hide a defect | Whatever each root cause replaces | done (on main). Each fix is at its root cause, and no limit was raised. Reader leases count the actual Parquet reader partitions, once per execution. The instrumented planner no longer records unbounded plan text (§23.1: Contract observation records spans only). The bench expectation was wrong: admission refuses rather than queues (ADR-0112, §14.3.2). Two further defects surfaced and are fixed: resident-cache reuse now survives an equivalent optimizer rebuild, and the M1 bench is moved onto `chemistry.*`. `bench-production` writes under `build/` and runs memory-capped. `just bench-smoke`, `bench-cache` and `bench-production` pass |
| H8 Memory regression in a PFR refinement study | `pfr_order_radau` (four PFRs of 5, 10, 20 and 40 Radau elements, about 2,000 variables) grew to 156 GB resident under a 64 GiB engine pool, twice, and the OS killed it, taking the editor down with it (2026-09-29 13:55, systemd-oomd; 14:20, kernel OOM). A problem this size needs a few GB, so the growth is an aberration introduced by recent code, not a missing accounting mechanism. Suspects: the CT-S05 `pse-modeling` change, KR1/KR2 inference, or combinatorial Symbolica expansion. Localize it by growth curve and commit bisection under the cap, and fix it at the root | Small refinement cases prepare with near-linear memory; a regression test bounds the growth; the full study runs within a few GB under the cap | The regression's cause | done (`e064d5c0`, on main). Root cause: since the Plan 21 projection, the compiler's modeling projection wrapped every output expression in a `let` holding every expression member, so memory grew as outputs × members inside the Salsa projection query, before any pool charge. It did not show until a Radau mesh larger than 4 elements existed. A single Radau PFR used 1.7 GB at 5 elements, 5.2 GB at 10 and 20.8 GB at 20. Each output now binds only the members it reads, in topological order: 0.43 GB at 5 elements and 0.52 GB at 10, and the whole-seed run peaks at 3.2 GB. Tests: `projected_outputs_bind_only_the_members_they_read`, `projection_size_grows_linearly_with_replicas` |
| H9 Retained-program reservations pile up; H8 | Within one conformance run, `math:native-job` leases accumulate at about 65 MiB each, mostly the default per-program foreign allowance, and a flash fixture holds about 200. After 88 fixtures, 121.6 GiB is reserved while RSS is 3.2 GB, so a later fixture is falsely refused and the rest go unattempted. Charge the foreign allowance while native work actually runs, or release retained programs under pool pressure, so reservations track real retention (DP-20) | A whole-seed run including the campaign package completes under a 48 GiB pool, with no false refusal; retained programs' reservations are bounded by what they actually retain | The per-program idle charge | done (on main). The foreign allowance is charged only while native work runs: a job while its thread runs, a native session while it is open, and a solve's declared allowance while its step runs. Retained products are charged their own size, and a program's size now includes its evaluators' instruction streams. On 94 fixtures the reservation after the last fixture fell from 104.6 GB to 6.9 GB. The seed passes 94/94 at the default pool and at a 48 GiB pool. Tests: `retained_programs_do_not_hold_the_foreign_allowance_while_idle`, `sequential_native_solves_keep_retained_reservations_bounded`, `evaluator_storage_counts_its_instruction_stream` |
| H10 A global infeasibility claim is checked against known feasible points | SCIP 10's convex nonlinear handler falsely proves the `bt_pr_liquid_stability` TPD infeasible: its cuts exclude the true liquid point. The track C agent reproduced this outside our code, on the exported problem. SCIP reports "proven infeasible", and nothing checks that claim. A certify or global result claiming infeasibility is compared with every known feasible candidate that satisfies the original constraints within tolerance (the local solution, a start, an evaluated incumbent). A contradicted claim becomes a typed, contradicted outcome, never infeasible (PS-10, PS-12) | `global_infeasibility_contradicted_by_a_known_feasible_point` against the saved reproducer; a genuinely infeasible model still reports infeasible | — | pending |
| H5 Typed expectations; KR6, KR7 | Expected failures match class plus lineage (parameter set, form, envelope layer, member) | `expected_envelope_failure_matches_set_layer_and_variable`, `expected_failure_with_wrong_lineage_fails_the_fixture` | Class-and-rule-only match | pending |
| H6 Parity report (CT-S14); KR6 | Oracle fixtures grouped by definition and oracle test with release, tolerance and disposition | `parity_report_lists_every_oracle_fixture_with_release_and_tolerance` | — | pending |

### Track C — process scenarios (fixtures in `packages/reference/campaign`)

| Packet | Responsibility | Acceptance | Deletion | Status |
|---|---|---|---|---|
| CT-S05 Steady → dynamic | `ControlVolume0D` generates holdup and `d/dt` under `analysis.dynamic`; a missing mechanism is a kernel gap | `cstr_integrated`, `cstr_simultaneous` unchanged; `cstr_steady_dynamic_same_definition` | Fixture-authored rate links and `cell[i in t]` children | done on `plan23/c` (`1d1d4bf3`), awaiting rebase. `ControlVolume0D` generates its holdup rates under `analysis.dynamic`, and one `reactors.CSTR` solves steady and dynamic (`cstr_steady_dynamic_same_definition`, `cstr_steady_same_definition`). Kernel gap: a derivative along a replicated coordinate reads its neighbour replicas (`replica_derivative_*`). Open decision: the flowsheet still replicates the unit over time (`child reactor[i in t]`); implicit lifting of lumped state over the analysis time domain would need a composition-semantics ADR |
| CT-S06 PFR order | Study over 5/10/20/40 elements, backward FD and Radau-3 | `pfr_order_backward`, `pfr_order_radau`; IDAES `test_pfr` TestInitializers oracle | — | done on `plan23/c` (`c5b2099e`), awaiting rebase. Backward FD observed order 0.913/0.955 over 5/10/20/40 elements. Radau-3 observed order 4.57/4.87 over 1/2/4/8 elements; at 20 and 40 the differences fall below solver tolerance and are refused by the typed opaque-derivative-support limit. `pfr_initializers_oracle` checks the IDAES `test_pfr` values |
| CT-S07 BT_PR formulation switch | `ComplementarityPR`, `NestedPRFlash`, `formulation` parameter; tangent-plane stability as `annotation check` | `bt_pr_flash_smooth`, `bt_pr_flash_complementarity`, `bt_pr_flash_nested` against IDAES `test_BT_PR` and `TestInitializersCubicModularBTX` | — | formulations done; rebasing onto main. Kernel gap: per-instance constraint-form realizations. `bt_pr_liquid_stability`: a real export defect is fixed, since SCIP rows are now exported in normalized coordinates (`64f3713f`). SCIP 10's convex nonlinear handler still falsely proves the model infeasible, reproduced outside our code. The fixture therefore asserts tpd = 0 and records a typed expected refusal with its reason (decision, register R-52; H10 adds the contradiction check). `--fixture` typed selection added (`76d563b8`) |
| CT-S09 Hard start | `RecycleFlash` (Feed → Mixer → Heater → Flash → Separator, liquid recycle tear, no value starts) | `recycle_flash_converges_from_defaults` over sampled feeds; `recycle_flash_failure_restores_specification` | — | pending |
| CT-S12 Costing | `HeatExchangerCost` reads the solved area of the BT co-current exchanger | `hx_costing_solved_area` (SSLW oracle, accounting closure) | Fixed 1000 m² fixture | pending |
| CT-S13 Diagnostics | Over-specified flash; minimal `EquilibriumCascade(n)` near total reflux | `flash_overspecified` (structural refusal naming members); `cascade_near_singular_diagnostics_name_members` | — | pending |

### Track D — domain schema and seed migration (sequential)

| Packet | Responsibility / dependencies | Acceptance | Deletion | Status |
|---|---|---|---|---|
| D0 Domain schema; KR4–KR8 | The schema above without rows (the chemistry kinds already moved in SM0); refusal corpus `tests/fixtures/domain-refusals/` | `domain_schema_refusals_name_the_violated_constraint` (duplicate CAS, unknown formula element, selection subject mismatch, missing pair selection, oracle row in a production root, conflicting symmetric pair rows, coefficient-unit mismatch, reaction element closure, apparent electroneutrality, cation charge); `derived_molar_mass`, `two_group_unifac_admission` | — | pending |
| SM1 Chemistry; D0 | `chem` splits into the `data/species` catalogue (CAS, InChIKey, formula, charge) and `data/ciaaw` elements; component sets move to the packages that bind them | `formula_weights` as a derived-attribute check | `atoms`, `atomic_mass` free tables | pending |
| SM2 Provenance as data; SM1 | Source entities (NIST Chase 1998, Perry 7 tables 2-196/2-30, RPP4, Poling 2000, Gross–Sadowski 2001, CIAAW 2024, IDAES 2.13 and its tests, teqp 0.23.1, FeOS 0.10.1, derived SciPy references); every dataset and test typed; reference scripts emit oracle Parquet | H1 unchanged | Every `source`/`revision` string, data prose in `sources.md`, `crates/pse-kernels/data/*`, `tests/fixtures/plan14/thermo-reference.json` once unread | provenance part done with KR6, on main: `pse.domain` with provenance kinds and roles, 57 source entities, typed sources on every seed dataset and seed-data test. Remaining: oracle Parquet generation, and deleting `crates/pse-kernels/data/*` and `thermo-reference.json` (after KR9) |
| SM3 Pure-component forms; SM2 | Forms `shomate`, `dippr100`, `dippr105`, `rpp4_cp`, `rpp4_wagner`, `antoine`, `constant`; increments `dh(T0,T)`, `ds(T0,T)`; rows to banks | Caloric (8) and phase-data (3) fixtures; indexed `∂dh/∂T == cp` over every caloric set | `fit` kind, `polynomial`/`scale`, species-keyed `shomate`, 13 name-encoded fits, unit helper functions, entropy-coordinate helpers, `PolynomialLiquid/Gas`, `liquid_density`/`vapor_pressure` tables | pending |
| SM4 Constants, reference states, EoS parameters; SM3 | Domain constants (R typed with the existing gas-constant kind); `cubic_family pr` and `kappa`; critical points as `oracle_input` sets with lineage; symmetric kᵢⱼ with package `missing zero`; PC-SAFT `sigma: Length`, constants keyed 0..6; named oracle reference states | eos (6), pcsaft (5), pr-oracle (3), homogeneous (4) fixtures unchanged | `gas_constant_value()`, PR digits, `6.02214076e-7`, pcsaft `pi()`, p0..p6/`power`, 13 zero kᵢⱼ rows, registry `reference.constants`, `enthalpy_datum()`, `stock_reference_temperature()` | pending |
| SM5 Property packages as selection; SM4 | `bt_ideal_idaes`, `bt_pr_idaes`, `saponification_idaes`, `vessel_light_hydrocarbons` with `selection` and roles; generic dispatching `component_enthalpy`, `psat`, `density`; phase slots; `component_role` | bt-ideal (16), bt-pr (2), unit (19), control (7), saponification (3) fixtures | Per-package property functions, binding tables, phase and solvent identity branches | pending |
| SM6 Reactions; SM5 | Stoichiometry relation with element and charge closure at admission; Arrhenius parameter sets | Saponification fixtures | Closure `expect`s in chemistry; saponification `stoichiometry` table | pending |

### Track S — scenarios on the domain model

| Packet | Responsibility | Acceptance | Status |
|---|---|---|---|
| CT-S01 Add a correlation | `rpp5_cp` and `antoine` (pressure unit and Celsius offset as typed attributes); water in `data/poling2000` | IDAES `test_RPP5` values (cp 33.518, h with and without formation, s 191.769, psat 3173.066); diff touches only `packages/` | pending |
| CT-S02 Add a unit | `flash_bt_ideal` on the domain binding | Recorded evidence | pending |
| CT-S03 Add an EoS | FeOS oracle bank (Parquet, `oracle_input`): p, h_res, s_res, cp_res, ln φ for the alkanes at five states | `pcsaft_properties_from_defaults_match_feos` | pending |
| CT-S10 Envelope | Form (Perry cp), data/package (BTIdeal 300–450 K) and closure (nested flash) failures | Matched with H5 lineage | pending |
| CT-S11 Local tests | Pure fixtures in H1 | No runtime or native startup | pending |
| DM1–DM6 | Domain scenarios above; DM1 `data/gross-sadowski-2001`, DM2 IDAES oracle and Poling 2000 Appendix A benzene, DM5 NRTL from IDAES `test_ideal_NRTL.py`, DM6 vessel fit | As defined above | pending |
| M1 Preparation scaling | H2 bench over DM1 and DM2 data | Measured; a typed refusal at 20 components is a truthful result | pending |
| M2 Flash convergence | 200 Latin-hypercube BT_PR feeds (Parquet case set), smooth versus nested flash | Measured success rate, iterations, time | pending |
| M3 Regime crossings | H4 counters over the M2 sample | Measured | pending |

### W5 — coordinator

| Packet | Responsibility | Acceptance | Status |
|---|---|---|---|
| AUD Knowledge-boundary audit | Reasoned review (`design-review`, `design-review-process-simulator`) of every `crates/` change in this plan: kernel gap with row and synthetic test, or violation; no scientific concept in Rust or the registry | Review recorded; no new lint or validator | pending |
| Q Qualification and closure | Verification below; Outcome; architecture owners; retirement | Actual results against zero | pending |

### Waves and coordination

| Wave | Track 1 | Track 2 | Track 3 | Track 4 |
|---|---|---|---|---|
| W0 | Coordinator: P0, H1, SM0 | — | — | — |
| W1 | K: KR1 → KR2 → KR3 | H: H2 → H3 → H4 | C: CT-S05, CT-S06, CT-S07 | — |
| W2 | K: KR4 → KR5 → KR6 → KR7 → KR9 | K′: KR8 | C: CT-S09, CT-S12, CT-S13 | H: H5, H6 |
| W3 | D: D0 → SM1 → … → SM6 | — | — | — |
| W4 | S-a: CT-S01, DM3, DM4 | S-b: DM1, DM2, CT-S03, M1 | S-c: DM5, DM6 | S-d: CT-S10, CT-S14, CT-S02/S11, M2, M3 |
| W5 | Coordinator: AUD, Q, Outcome, documentation, retirement | — | — | — |

Worktrees per track, at most four compiling at once, each with its own target directory.
One track at a time edits `pse-schema`, the root `Cargo.toml` or `Cargo.lock`; track K
holds that token by default. After each merge the coordinator runs `just codegen` (and
`just python-stubs` where needed; generated conflicts are regenerated), reruns the
packet's targeted tests on `main` without concurrent builds, runs H1 after package-data
changes and records a checkpoint below. Each wave ends with the architecture text for the
mechanisms it landed, under `PSE_DESIGN_EDIT=1`, with one blueprint revision row.

Critical path: KR1 → KR9, then D0 → SM6.

## Finding dispositions

Transferred from Plan 21 on 2026-09-29. K8 evidence alone closes none of them; each
closes with the acceptance of the named scenarios.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f01) provider fuses model and algorithm | CT-S03 | scheduled | CT-S03 | — |
| [F02](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f02) no demand derivation | CT-S01, CT-S02 | scheduled | CT-S01, CT-S02 | — |
| [F03](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f03) method data never executes | CT-S01, CT-S02 | scheduled | SM3, CT-S01 | — |
| [F04](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f04) scalar-only laws | CT-S02, CT-S12 | scheduled | CT-S02, CT-S12 | — |
| [F05](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f05) no formulation primitives | CT-S07 | scheduled | CT-S07 | — |
| [F06](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f06) composition cannot express the library | CT-S02, CT-S06 | scheduled | CT-S02, CT-S06 | — |
| [F07](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f07) semantics in the runtime | CT-S08, CT-S11 | scheduled | H3, CT-S11 | — |
| [F08](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f08) steady and dynamic authored apart | CT-S05 | scheduled | CT-S05 | — |
| [F09](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f09) no continuous domains | CT-S06 | scheduled | CT-S06 | — |
| [F10](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f10) no initialization knowledge | CT-S09 | scheduled | CT-S09 | — |
| [F11](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f11) no derived nominals | CT-S08 | scheduled | H3 | — |
| [F12](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f12) diagnostics incomplete | CT-S13 | scheduled | CT-S13 | — |
| [F13](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f13) FeOS cannot own the scope | CT-S03 | scheduled | CT-S03, SM2 | — |

The [typed-domain review](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md)
findings are resolved in the text of ADR-0123 to ADR-0125 and ADR-0127 (review §13; §13.1 corrects F07). Each closes when
its implementation evidence exists:

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [Typed-domain F01](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f01) source revision preimage | DM1, S01 | scheduled | KR3, KR9 | revision-identity tests (ADR-0123 Outcome 8) |
| [Typed-domain F02](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f02) chain leaf eligibility and result policy | CT-S01, DM3 | scheduled | KR2 | `seed_forms_type_without_intermediate_kinds`, `nonzero_datum_leaf_is_refused_with_its_factor` |
| [Typed-domain F03](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f03) keyed identity | DM2 | scheduled | KR4 | `keyed_identity_is_the_key_declaring_kind_and_typed_keys`, `same_key_in_two_forms_is_refused` |
| [Typed-domain F04](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f04) test-only taint granularity | DM4 | scheduled | KR6 | `production_root_reading_an_oracle_row_is_refused`, `shared_relation_with_oracle_rows_keeps_production_roots_admissible` |
| [Typed-domain F05](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f05) admission order and self-reference | DM6 | scheduled | KR5 | `self_referential_lineage_admits_and_cycles_are_refused` |
| [Typed-domain F06](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f06) pair and phase-model selection | DM2, DM5 | scheduled | D0 | missing pair selection refusal; DM2 pairs |
| [Typed-domain F07](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f07) chemistry kinds' placement | — | scheduled | SM0, ADR-0127 | shaped types deleted; subject kinds stay in `pse.physical` (review §13.1) |
| [Typed-domain F08](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f08) datum stated twice | CT-S10 | scheduled | D0, KR8 | parameter sets carry no reference attribute |
| [Typed-domain F09](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f09) binary documents across text contracts | DM1 | scheduled | KR9 | `durable_job_round_trips_a_package_with_a_data_document`, `parquet_metadata_unit_disagreement_is_refused` |
| [Typed-domain F10](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f10) admitted-table reuse owner | S01 | scheduled | KR9 | `unchanged_dataset_reuses_admitted_table` (incremental equals clean) |
| [Typed-domain F11](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f11) extrapolation owner and increment guards | CT-S10 | scheduled | KR7 | `increment_guards_its_integration_interval`, `consumer_selects_extrapolation_per_layer` |
| [Typed-domain F12](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f12) closed role taxonomy | DM4 | scheduled | KR6 | `new_role_is_a_package_only_edit` |
| [Typed-domain F13](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f13) static `Ref` precondition | CT-S01 | scheduled | KR4 | `non_static_ref_is_refused` |
| [Typed-domain F14](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f14) unit-product identity | — | scheduled | KR1 | `unit_product_is_order_independent`, `report_in_a_composite_unit_carries_its_identity` |
| [Typed-domain F15](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f15) gas-constant kind | — | scheduled | SM4 | eos and homogeneous fixtures unchanged (H1) |

## Verification

Targeted checks accompany each packet: `just unit-package <pkg> '<filter>'`,
`just native-test -E '…'` for solver-backed checks, `just seed-conformance` after
package-data packets, `just codegen`, `just python-stubs` with targeted
`just py-unit-native` for Python-visible changes, and `just adr-lint` for decisions.
No formatting, lint or integration suites run mid-plan.

Q reports each check against the zero baseline with its command and conditions:
`just fmt-check`, `just clippy`, `just lint-solver-contracts`, `just quality`,
`just governance`, `just codegen-check`, `just adr-lint`, `just docs`, `just docs-test`,
offline link checking, `just test`, `just native-test`, `just doctest`,
`just native-python`, `just publication-test`, `just worker-test`, `just parity`,
`just seed-conformance` (every fixture in one run, pool recorded), `just bench-production`
and `just case-measure` for M1–M3 and 10⁵-row admission, and the AUD review.

## Open items

- Selection rests on static `Ref`s: a call through a `Ref` specializes per resolved
  most-derived function on the existing function-specialization path, and a `Ref` that
  depends on a runtime value is refused (KR4). `Fn`-typed table columns already specialize
  per selected function and remain the fallback.
- Increment forms (`dh`, `ds`) may expose cancellation against 1e-10 identity
  tolerances; tolerances are not relaxed.
- M1 at 20 components may exceed memory; that is a Measured refusal, not a blocker.
- Whole-seed pools (64–128 GiB) are recorded as run conditions.

## Current checkpoint

**2026-09-29.** Main (`5664e80a`) carries:
- P0, H1, H1f and ADR-0127;
- SM0 (chemistry kinds in `pse.physical`'s `chemistry` module; 354 unreferenced shaped
  types deleted);
- KR1–KR2 (unit products and derived kinds, including the entropy increment);
- H2–H4 (lazily extended formal pool, study preparation counts, typed regime crossings);
- the memory guard `scripts/memory-cap.sh`.

Verified on main's tree:
- `just seed-conformance`: 94/94 passed, complete (128 GiB pool, memory-capped);
- targeted `just native-test`: 73 passed, 0 failed;
- the seven kernel crates' library suites: 0 failures.

Two editor-wide OOM crashes were traced to one runaway run and are tracked as H8:
- the track C study `pfr_order_radau` grew to 156 GB resident;
- a problem of that size needs a few GB, so H8 is a regression hunt in recent code.

In progress:
- H8, then CT-S06 and CT-S07 (`plan23/c`; CT-S05 at `1d1d4bf3`);
- KR3 + KR8 (`plan23/kr2`); the structured IR takes `authored.modeling_declarations`
  version 9, because H1f took version 8.

Next: KR4–KR7 and KR9; H5, H6 and H7; then D0 and SM1–SM6.

Integration keeps history linear from here: track branches are rebased onto main.

Decisions during execution:
- The `chem` split into data-bank distributions moves to SM1.
- The declaration-ID fixture subset in `tests/support/plan14.rs` stays.
- ADR-0127 supersedes ADR-0126. The chemical core stays in `pse.physical`, because physical
  quantity types name its kinds as subjects (register R-51).
- Entropy increments type through declared dimensionless kinds, not through expected-type
  selection among same-monomial kinds.
- Automatic presolve declines only the offending interval propagation (H1f).

KR3 + KR8 are on main, rebased linearly. Verified on main's tree:
- `just check` clean, and regeneration makes no further changes;
- the seven kernel crates' library suites: 426 passed;
- `just seed-conformance`: 94/94;
- targeted `just native-test`: 75 passed, 0 failed.
- `authored.modeling_declarations` version 9: the type arena, enums, typed imports and
  fact namespaces.
- New frame variants:
  - `ModelingSourceRevisionV2`, whose preimage covers the structured rows, the physical
    inventory identity and the physical name bindings the source resolves with;
  - `ModelingDispatchBodyV2`, `ModelingFiniteFunctionV2`, `ModelingContinuityV2`,
    `ModelingDefiniteIntegralV2`, `ModelingConsumerBodyV2`, `ModelingImplicitResidualV2`,
    `MathTypedDefinitionV3` and `MathPhysicalInventoryV5`.
- Quantity types and reference states are named in the physical document. The
  reference-state names are `package_datum`, `stock`, `bt_ideal_oracle` and `bt_pr_oracle`.
  Package dependencies are typed.
- The `identifier` type node parses; resolution refuses it until KR4 declares identifier
  schemes.

Architecture text for SM0, KR1–KR3, KR8, H1, H1f, H2–H4 and H7 is written (blueprint revisions 78, 80 and 81; `14a3e834`).

KR4 is on main. Verified on main's tree: seed 94/94; targeted native tests 75/75; kernel
suites 442 passed; linked Python 22 passed.

Syntax ratified (maintainer's plan authorization; coordinator decision):
- a kind's name is its reference type, so there is no separate `Ref<k>` spelling;
- identifiers are written `Id<scheme>("value")`;
- uncertainty is written `± standard|relative|bound(x)`;
- a dataset declares supplied keys with `bind(key = cell)`;
- a refined kind binds an inherited attribute with `name = cell;`.

KR5 is on main. Verified on main's tree: seed 94/94; targeted native tests 75/75; kernel
suites 477 passed; linked Python 22 passed.

Kernel gaps D0 must close (KR5 supports these on tables only):
- kind-level derived attributes (for example `derived molar_mass`);
- kind-level `require`;
- attribute-level `unique`. Identifier attributes are already unique within the closure
  (KR4).

The language keyword stays `table`; the sketch's `relation` is illustrative.

KR6 + SM2 provenance are on main. Verified on main's tree: seed 94/94; targeted native
tests 79/79; kernel and runtime suites 527 passed; linked Python 22 passed.
- `authored.modeling_declarations` is at version 12 and `runtime.modeling_conformance` at
  version 2 (`oracle_source_id`). The typed test-only refusal is rule
  `modeling.provenance`.
- `pse.domain` exists with its provenance kinds and roles, plus 57 source entities.
- All 34 seed datasets and all 91 seed-data tests carry typed sources.

Decisions (coordinator, under the maintainer's plan authorization):
- **Deliberate deviation from ADR-0123 Outcome 5 wording.** A test names an optional
  oracle source and no role. The three `pse.physical` tests cite none, because
  `pse.domain` sits above `pse.physical`.
- **Lineage kinds are `dataset` and `source` for now.** `fit` lineage arrives with DM6 and
  register R-50.
- **The added kinds `release_artifact` and `computation` are accepted.**
- **Lineage taints.** Data derived from test-only inputs is test-only, so a `derived`
  dataset cannot launder oracle data into production. KR7 carries this correction.

KR7 is on main. Verified on main's tree: seed 94/94; targeted native tests 91/91; kernel
and runtime unit suites 531 passed; linked Python 23 passed.

Decisions (coordinator):
- The closure layer's policy stays on `annotation valid`, and the data layer is selected by
  `extrapolation data …` in a package, test or case scope. Each layer's policy has one owner.
- The nearest selection wins.
- Extrapolation is allowed only where it can be recorded.
- Row bounds must be ordered.

Finding for later: static evaluation still skips domain and envelope checks. This predates
Plan 23. Owner: H5, or its own packet.

Carried into KR5 (resolved):
- A cell can reference declared entities but not keyed rows. KR5 adds a typed keyed-row
  reference cell, a kind plus a key tuple resolved at admission, which `selection` needs.
- The Python contract generator orders nested named structures alphabetically. It raised
  a NameError, so KR4 left the unit factor structure unnamed. Fix the generator's
  dependency order.

Architecture text owed:
- §6.1, §6.2, §6.15.1: version 10 cells, attributes, keys, bindings, identifier schemes,
  constants, dataset targets and bindings, and member identities;
- §5.1, §5.3: keyed identity and the three new frames.

## Outcome (recorded after implementation)

### What was built

### A mistake made and corrected

### Deviations from the plan, deliberate
