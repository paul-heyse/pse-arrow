// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
fn witness() -> SemanticContract {
    let registry = crate::registry().unwrap();
    SemanticContract::new(
        registry,
        &[registry.relation("runtime.solve_runs").unwrap().id].into(),
    )
    .unwrap()
}
fn recorded(value: SemanticContract) -> Result<VerifiedRecordedContract, CompatibilityError> {
    verify_recorded(
        Some(FORMAT),
        Some(&pse_columnar::native_field::canonical_json(&value).unwrap()),
        value.identity().unwrap(),
    )
}
#[test]
fn directional_enum_growth_does_not_authorize_an_exact_write() {
    let old = witness();
    let root = *old.roots.iter().next().unwrap();
    let name = old.relations[&root]["enums"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    let mut new = old.clone();
    for description in new.relations.values_mut() {
        if let Some(domain) = description["enums"].get_mut(&name) {
            domain["members"]["future_member"] = serde_json::json!(false);
        }
    }
    let old = recorded(old).unwrap();
    let new = recorded(new).unwrap();
    old.project(new.contract()).unwrap();
    assert!(matches!(
        new.project(old.contract()),
        Err(CompatibilityError::Incompatible(_))
    ));
    assert!(matches!(
        old.admit_exact_write(new.contract(), ContentHash::NIL, ContentHash::NIL),
        Err(CompatibilityError::Incompatible(_))
    ));
    old.admit_exact_write(old.contract(), ContentHash::NIL, ContentHash::NIL)
        .unwrap();
}
#[test]
fn deprecation_is_metadata_but_constraints_and_references_are_consumed_meaning() {
    let original = witness();
    let root = *original.roots.iter().next().unwrap();
    let mut changed = original.clone();
    for description in changed.relations.values_mut() {
        for domain in description["enums"].as_object_mut().unwrap().values_mut() {
            for deprecated in domain["members"].as_object_mut().unwrap().values_mut() {
                *deprecated = serde_json::json!(true);
            }
        }
    }
    let old = recorded(original).unwrap();
    old.project(recorded(changed).unwrap().contract()).unwrap();
    let mut constraints = old.contract().clone();
    constraints.relations.get_mut(&root).unwrap()["checks"]["new_constraint"] =
        serde_json::json!(crate::fingerprint::canonical_sql("TRUE", true).unwrap());
    assert!(
        old.project(recorded(constraints).unwrap().contract())
            .is_err()
    );
}
#[test]
fn recorded_verification_rejects_self_consistent_missing_or_unreachable_support() {
    let registry = crate::registry().unwrap();
    let original = SemanticContract::new(
        registry,
        &[registry.relation("authored.entities").unwrap().id].into(),
    )
    .unwrap();
    let mut missing = original.clone();
    let support = *missing
        .relations
        .keys()
        .find(|id| !missing.roots.contains(id))
        .unwrap();
    missing.relations.remove(&support);
    assert!(matches!(
        recorded(missing),
        Err(CompatibilityError::Malformed(_))
    ));
    let mut wrong_id = original.clone();
    let root = *wrong_id.roots.iter().next().unwrap();
    wrong_id.relations.get_mut(&root).unwrap()["id"] = serde_json::json!(SemanticId::NIL);
    assert!(matches!(
        recorded(wrong_id),
        Err(CompatibilityError::Malformed(_))
    ));
    let verified = recorded(original).unwrap();
    assert!(
        verify_recorded(
            Some(FORMAT),
            Some(&pse_columnar::native_field::canonical_json(verified.contract()).unwrap()),
            ContentHash::NIL
        )
        .is_err()
    );
    assert!(matches!(
        verify_recorded(None, None, ContentHash::NIL),
        Err(CompatibilityError::MigrationRequired(_))
    ));
}
#[test]
fn every_current_root_verifies_without_registry_backed_recorded_interpretation() {
    let registry = crate::registry().unwrap();
    for relation in registry.relations() {
        let value = SemanticContract::new(registry, &[relation.id].into()).unwrap();
        let recorded = recorded(value).unwrap_or_else(|e| panic!("{}: {e}", relation.key));
        let schema = crate::arrow::relation_schema(registry, relation).unwrap();
        let encoding = crate::fingerprint::encoding_relation(registry, relation).unwrap();
        recorded
            .verify_fields(relation.id, &schema, encoding)
            .unwrap_or_else(|e| panic!("{} fields: {e}", relation.key));
    }
}
#[test]
fn consumer_selection_ignores_unconsumed_roots_and_preserves_original_digest() {
    let registry = crate::registry().unwrap();
    let a = registry.relation("runtime.solve_runs").unwrap().id;
    let b = registry
        .relation("runtime.artifact_descriptors")
        .unwrap()
        .id;
    let source = recorded(SemanticContract::new(registry, &[a, b].into()).unwrap()).unwrap();
    let identity = source.identity();
    let consumer = source.select_roots(&[a].into()).unwrap();
    source.project(consumer.contract()).unwrap();
    assert!(consumer.contract().roots.contains(&a));
    assert_eq!(source.identity(), identity);
}

#[test]
fn empty_contract_is_valid_only_with_empty_support() {
    let empty = SemanticContract {
        version: SEMANTIC_VERSION,
        roots: Default::default(),
        relations: Default::default(),
    };
    recorded(empty).unwrap();
    let mut invalid = witness();
    invalid.roots.clear();
    assert!(matches!(
        recorded(invalid),
        Err(CompatibilityError::Malformed(_))
    ));
}

#[test]
fn recorded_versions_remain_distinct_and_each_root_selects_its_exact_declaration() {
    use crate::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};
    let mut builder = crate::RegistryBuilder::new();
    for version in [1, 2] {
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "versioned_values",
                version,
                Authority::Authored,
                SnapshotClass::Model,
                "Independently selectable relation versions.",
            )
            .pk(&["id"])
            .foreign_key("self", &["id"], "authored.versioned_values", &["id"])
            .columns(vec![
                FieldContract::key(
                    "id",
                    FieldContract::native(arrow_schema::DataType::Int64),
                    "Row key.",
                )
                .with_fk("authored.versioned_values", "id"),
            ]),
        );
    }
    let registry = builder.build().unwrap();
    let roots = registry.relations().iter().map(|spec| spec.id).collect();
    let source = recorded(SemanticContract::new(&registry, &roots).unwrap()).unwrap();
    assert_eq!(source.contract().roots.len(), 2);
    for spec in registry.relations() {
        let independently_recorded =
            recorded(SemanticContract::new(&registry, &[spec.id].into()).unwrap()).unwrap();
        let selected = source.select_roots(&[spec.id].into()).unwrap();
        assert_eq!(independently_recorded.contract(), selected.contract());
        assert_eq!(selected.contract().relations.len(), 1);
        assert_eq!(
            selected.contract().relations[&spec.id]["foreign_keys"]["self"]["target"],
            serde_json::json!(spec.id)
        );
        assert_eq!(
            selected.contract().relations[&spec.id]["relation"],
            serde_json::json!(spec.key.to_string())
        );
        let mut corrupt = selected.contract().clone();
        corrupt.relations.get_mut(&spec.id).unwrap()["relation"] =
            serde_json::json!("authored.versioned_values@3");
        assert!(matches!(
            recorded(corrupt),
            Err(CompatibilityError::Malformed(_))
        ));
    }
}
#[test]
fn missing_field_projection_is_only_the_declared_historical_descriptor_rule() {
    let registry = crate::registry().unwrap();
    let id = registry
        .relation("runtime.artifact_descriptors")
        .unwrap()
        .id;
    let target = SemanticContract::new(registry, &[id].into()).unwrap();
    let mut old = target.clone();
    old.relations.get_mut(&id).unwrap()["fields"]
        .as_array_mut()
        .unwrap()
        .retain(|field| field["name"] != "profile_required_relations");
    let old = recorded(old).unwrap();
    let projection = old.project(&target).unwrap();
    assert_eq!(
        projection
            .field_slots(id)
            .unwrap()
            .iter()
            .filter(|slot| slot.is_none())
            .count(),
        1
    );
    let mut unrelated_missing = target.clone();
    unrelated_missing.relations.get_mut(&id).unwrap()["fields"]
        .as_array_mut()
        .unwrap()
        .retain(|field| field["name"] != "value_assumptions");
    assert!(
        recorded(unrelated_missing)
            .unwrap()
            .project(&target)
            .is_err()
    );
}

#[test]
fn unconsumed_field_extension_growth_does_not_invalidate_a_reader() {
    let registry = crate::registry().unwrap();
    let relation = registry.relation("runtime.solve_runs").unwrap();
    let consumer = SemanticContract::new(registry, &[relation.id].into()).unwrap();
    let mut expanded = relation.clone();
    expanded.columns.push(
        crate::model::FieldContract::extended(crate::model::ExtensionUse::Bound)
            .with_name("unconsumed_bound"),
    );
    let mut source = consumer.clone();
    source.relations.insert(
        relation.id,
        crate::fingerprint::semantic_description(registry, &expanded).unwrap(),
    );
    recorded(source).unwrap().project(&consumer).unwrap();
}

#[test]
fn unavailable_feasibility_preserves_recorded_candidate_vocabulary() {
    let registry = crate::registry().unwrap();
    let root = registry
        .relation("runtime.candidate_assessments")
        .unwrap()
        .id;
    let current = SemanticContract::new(registry, &[root].into()).unwrap();
    let mut historical = current.clone();
    let mut removed = 0;
    for description in historical.relations.values_mut() {
        if let Some(domain) = description["enums"].get_mut("CandidateRefusal") {
            assert!(
                domain["members"]
                    .as_object_mut()
                    .unwrap()
                    .remove("feasibility_unavailable")
                    .is_some()
            );
            removed += 1;
        }
    }
    assert!(removed > 0);
    let historical = recorded(historical).unwrap();
    let current = recorded(current).unwrap();
    historical.project(current.contract()).unwrap();
    assert!(current.project(historical.contract()).is_err());
    assert!(
        historical
            .admit_exact_write(current.contract(), ContentHash::NIL, ContentHash::NIL)
            .is_err()
    );
    historical
        .admit_exact_write(historical.contract(), ContentHash::NIL, ContentHash::NIL)
        .unwrap();
    let routes = registry.relation("runtime.route_decisions").unwrap();
    assert_eq!(routes.key.version, 2);
}
