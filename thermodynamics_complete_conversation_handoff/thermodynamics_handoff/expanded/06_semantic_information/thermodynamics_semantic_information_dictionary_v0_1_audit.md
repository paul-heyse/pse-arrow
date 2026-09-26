# THERMO-INFORMATION-006 v0.1 audit

| Check | Pass |
| --- | --- |
| 72_unique_concepts | True |
| concept_fields_complete | True |
| unique_fields_per_concept | True |
| cardinalities_valid | True |
| one_valid_semantic_owner_per_concept | True |
| field_concept_references_valid | True |
| relationship_endpoints_cardinalities_valid | True |
| unique_relationship_ids | True |
| 94_requirement_ids_preserved | True |
| primary_concept_owner_matches_step5 | True |
| all_concepts_have_requirement_trace | True |
| requirement_classes_preserved | True |
| original_requirement_test_links_preserved | True |
| 18_information_products_preserved | True |
| product_owners_preserved | True |
| product_primary_concept_owner_matches | True |
| product_decomposition_owners_match | True |
| every_concept_in_exactly_one_product_decomposition | True |
| 34_scenarios_and_profiles_preserved | True |
| scenario_information_routes_present | True |
| all_rules_have_valid_concepts_and_witnesses | True |
| all_examples_have_valid_concepts_and_scenarios | True |
| all_P3_scenarios_have_concrete_examples | True |
| step5_open_item_owners_preserved | True |
| source_files_present | True |
| step5_recorded_source_hashes_match | True |
| step4_recorded_source_hashes_match | True |
| authored_arithmetic_and_guard_logic_checks_pass | True |
| all_concepts_have_positive_negative_examples | True |
| unchanged_step5_contract_graph_acyclic | True |

| Count | Value |
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

## Authored arithmetic / logical checks only

| Name | Actual | Expected | Pass |
| --- | --- | --- | --- |
| PH target formula | 50 | 50 | True |
| False-success enthalpy residual | 30 | 30 | True |
| Two-liquid A total | 5 | 5 | True |
| Two-liquid B total | 5 | 5 | True |
| Binary reference shift z=(1/2,1/2) | 25 | 25 | True |
| Binary reference shift z=(3/4,1/4) | 35/2 | 35/2 | True |
| Reaction reference delta at extent 0.4 | 12 | 12 | True |
| Residual model discrepancy after stated offset | 10 | 10 | True |
| Temperature for corrected H=50 | 1025/3 | 1025/3 | True |
| Frozen versus response slope difference | 1 | 1 | True |
| Constrained composition derivative x1 | -3 | -3 | True |
| Constrained composition derivative x2 | -2 | -2 | True |
| Empirical heating duty | 100 | 100 | True |
| Mass-weighted histogram first bin | 7/20 | 7/20 | True |
| Mass-weighted histogram second bin | 13/20 | 13/20 | True |
| Surface amount at stated loading | 1 | 1 | True |
| Loading after transfer | 11/50 | 11/50 | True |
| Bulk plus surface conservation | 3 | 3 | True |
| Illustrative publication conjunction True/True/True | True | True | True |
| Illustrative publication conjunction False/True/True | False | False | True |
| Illustrative publication conjunction True/False/True | False | False | True |
| Illustrative publication conjunction True/True/False | False | False | True |

## Limitations

These checks concern authored catalog structure, referenced baseline preservation, and exact synthetic arithmetic/Boolean conjunctions.

No actual simulator, provider integration, thermodynamic solver, derivative engine, fault-injection harness, concurrent publication implementation or empirical model was tested.

No inherited acceptance-test status or scenario R/E/V status was upgraded.

Relationships between concepts are semantic associations, not a claim of acyclic implementation imports.
