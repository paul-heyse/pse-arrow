// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Production P3/P4/P5 fixtures use actual authoring, shared DataFusion and admitted stages.
#[path = "support/demand_source.rs"]
mod demand_source;
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;
use demand_source::id;
use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package_texts},
};
use pse_schema::{Registry, model::Cell};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn changed(
    registry: &Registry,
    change: impl FnOnce(&mut BTreeMap<String, String>),
) -> DocumentBundle {
    let mut texts = demand_source::source(registry, false)
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect();
    change(&mut texts);
    load_package_texts(texts, registry, ParseBudget::default()).unwrap()
}
fn edit(texts: &mut BTreeMap<String, String>, path: &str, change: impl FnOnce(&mut Value)) {
    let mut value: Value = serde_json::from_str(&texts[path]).unwrap();
    change(&mut value);
    texts.insert(path.to_owned(), value.to_string());
}
fn rows(registry: &Registry, stage: &pse_catalog::Snapshot, name: &str) -> Vec<Vec<Cell>> {
    let spec = registry.relation(name).unwrap();
    let batch = stage
        .relation(spec.key.namespace.as_str(), spec.key.name)
        .unwrap()
        .batch();
    pse_relations::cells::cells_from_batch(registry, spec, batch).unwrap()
}
fn get<'a>(registry: &Registry, name: &str, row: &'a [Cell], field: &str) -> &'a Cell {
    &row[registry
        .relation(name)
        .unwrap()
        .columns
        .iter()
        .position(|column| column.name() == field)
        .unwrap()]
}
fn selector(
    term: u8,
    scope: u8,
    parent: Option<u8>,
    ordinal: u16,
    op: &str,
    entity: Option<u8>,
) -> Value {
    json!({"term_id":id(term),"scope_id":id(scope),"parent_term_id":parent.map(id),"ordinal":ordinal,"op":op,"entity_id":entity.map(id),"entity_kind":null,"tag":null,"parameter_name":null})
}

#[tokio::test]
async fn relative_parameter_selector_tracks_actual_target_and_refuses_missing_instance() {
    for target in [70, 71, 200] {
        let mut fixture = native_pipeline::Fixture::new();
        let semantic_id_type = fixture.registry.logical_type("semantic_id").unwrap().id;
        let document = changed(&fixture.registry, |texts| {
            edit(texts, "templates/state.yaml", |value| {
                value["template_params"] = json!([{
                    "template_id": id(60), "name": "selected_instance",
                    "logical_type_id": semantic_id_type, "enum_id": null,
                    "default": id(target).to_hex(), "required": true,
                    "domain_spec": null, "doc": "Explicit actual selector target"
                }]);
            });
            edit(texts, "instances/state.yaml", |value| {
                value["instances"].as_array_mut().unwrap().push(json!({
                    "id": id(71), "template_id": id(62), "name": "other",
                    "param_values": [], "feature_values": [], "doc": "Distinct target"
                }));
                value["selector_terms"][0]["op"] = json!("instance_parameter");
                value["selector_terms"][0]["parameter_name"] = json!("selected_instance");
            });
        });
        let model = fixture.commit(vec![document]).await;
        let result = fixture.run(model, "P5").await;
        if target == 200 {
            assert!(
                result.is_err(),
                "an absent configured instance is an explicit failed obligation"
            );
            continue;
        }
        let report = result.unwrap();
        let stage = &report.stages.last().unwrap().snapshot;
        let scopes = rows(&fixture.registry, stage, "inferred.scope_bindings");
        let binding = scopes
            .iter()
            .find(|row| {
                get(
                    &fixture.registry,
                    "inferred.scope_bindings",
                    row,
                    "scope_decl_id",
                ) == &Cell::Id(id(65))
                    && get(
                        &fixture.registry,
                        "inferred.scope_bindings",
                        row,
                        "owner_instance_id",
                    ) == &Cell::Id(id(70))
            })
            .unwrap();
        let scope = get(
            &fixture.registry,
            "inferred.scope_bindings",
            binding,
            "scope_id",
        );
        let members = rows(&fixture.registry, stage, "inferred.scope_members")
            .into_iter()
            .filter(|row| &row[0] == scope)
            .map(|row| row[1].clone())
            .collect::<Vec<_>>();
        assert_eq!(members, vec![Cell::Id(id(target))]);
        assert!(
            stage
                .relation("provenance", "selector_parameter_target_assertions")
                .unwrap()
                .batch()
                .num_rows()
                > 0
        );
    }
}

#[tokio::test]
async fn scalar_child_guards_and_nested_selector_difference_use_actual_instance_keys() {
    for enabled in [true, false] {
        let mut fixture = native_pipeline::Fixture::new();
        let document = changed(&fixture.registry, |texts| {
            edit(texts, "templates/state.yaml", |value| {
                value["template_guards"] = json!([{"guard_id":id(140),"template_id":id(60),"predicate":enabled.to_string(),"doc":"actual scalar child decision"}]);
                value["template_submodels"] = json!([{"template_id":id(60),"name":"child","child_template_id":id(61),"child_from_param":null,"multiplicity_domain":null,"bindings":[],"guard_id":id(140)}]);
            });
            edit(texts, "instances/state.yaml", |value| {
                value["instances"].as_array_mut().unwrap().push(json!({"id":id(71),"template_id":id(62),"name":"other","param_values":[],"feature_values":[],"doc":"independent actual instance"}));
                value["scopes"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"scope_id":id(150),"root_term_id":id(151)}));
                value["selector_terms"].as_array_mut().unwrap().extend([
                    selector(151, 150, None, 0, "difference", None),
                    selector(152, 150, Some(151), 0, "union", None),
                    selector(153, 150, Some(152), 0, "include", Some(70)),
                    selector(154, 150, Some(152), 1, "include", Some(71)),
                    selector(155, 150, Some(151), 1, "include", Some(71)),
                ]);
            });
        });
        let model = fixture.commit(vec![document]).await;
        let report = fixture.run(model, "P5").await.unwrap();
        let stage = &report.stages.last().unwrap().snapshot;
        let instances = rows(&fixture.registry, stage, "inferred.instances");
        let child = instances
            .iter()
            .filter(|row| {
                get(
                    &fixture.registry,
                    "inferred.instances",
                    row,
                    "parent_instance_id",
                ) == &Cell::Id(id(70))
            })
            .collect::<Vec<_>>();
        assert_eq!(child.len(), usize::from(enabled));
        if enabled {
            let tree = rows(&fixture.registry, stage, "inferred.instance_tree");
            assert!(tree.iter().any(|row| get(
                &fixture.registry,
                "inferred.instance_tree",
                row,
                "ancestor_id"
            ) == &Cell::Id(id(70))
                && get(&fixture.registry, "inferred.instance_tree", row, "depth")
                    == &Cell::U64(1)));
        }
        let members = rows(&fixture.registry, stage, "inferred.scope_members");
        let selected = members
            .iter()
            .filter(|row| row[0] == Cell::Id(id(150)))
            .map(|row| row[1].clone())
            .collect::<Vec<_>>();
        assert_eq!(selected, vec![Cell::Id(id(70))]);
        assert!(
            stage
                .relation("provenance", "selector_decision_assertions")
                .unwrap()
                .batch()
                .num_rows()
                > 0
        );
    }
}

#[tokio::test]
async fn unresolved_and_conflicting_feature_assignments_refuse_actual_guarded_expansion() {
    for conflict in [false, true] {
        let mut fixture = native_pipeline::Fixture::new();
        let document = changed(&fixture.registry, |texts| {
            edit(texts, "templates/state.yaml", |value| {
                value["template_features"] = json!([
                    {"template_id":id(60),"name":"enabled","kind":"bool","enum_id":null,"default":if conflict {Some("true")} else {None},"inherit_from":null,"doc":"explicit knowledge state"},
                    {"template_id":id(60),"name":"required","kind":"bool","enum_id":null,"default":"false","inherit_from":null,"doc":"incompatible assignment in conflict case"}
                ]);
                if conflict {
                    value["template_feature_rules"] = json!([{"template_id":id(60),"rule":"implies","antecedent":"enabled","consequent":"required"}]);
                }
                value["template_guards"] = json!([{"guard_id":id(140),"template_id":id(60),"predicate":"enabled","doc":"actual feature read"}]);
                value["template_submodels"] = json!([{"template_id":id(60),"name":"child","child_template_id":id(61),"child_from_param":null,"multiplicity_domain":null,"bindings":[],"guard_id":id(140)}]);
            })
        });
        let model = fixture.commit(vec![document]).await;
        assert!(
            fixture.run(model, "P5").await.is_err(),
            "unknown and conflict cannot become arbitrary false/true child decisions"
        );
    }
}

#[tokio::test]
async fn cyclic_parallel_connections_keep_exact_interfaces_and_finite_tear_witnesses() {
    let mut fixture = native_pipeline::Fixture::new();
    let port = |owner| pse_ids::named_id(id(owner), "port:stream");
    let document = changed(&fixture.registry, |texts| {
        edit(texts, "templates/state.yaml", |value| {
            value["template_port_members"] = json!([{"template_id":id(60),"ordinal":0,"symbol_decl_id":id(40),"symbol_group":"requested"}]);
            value["template_ports"] = json!([{"template_id":id(60),"name":"stream","kind":"material","direction":"bidirectional","bound_to":"self","guard_id":null,"doc":"explicit scalar interface"}]);
            value["connection_bindings"] =
                json!([{"rule_template_id":id(63),"expansion":"equality"}]);
        });
        edit(texts, "instances/state.yaml", |value| {
            value["instances"].as_array_mut().unwrap().push(json!({"id":id(71),"template_id":id(60),"name":"second","property_package_id":id(95),"param_values":[],"feature_values":[],"doc":"second actual state"}));
            value["connections"] = json!([
                {"connection_id":id(160),"from_port_id":port(70),"to_port_id":port(71),"rule_template_id":id(63),"tear_cost":2.0,"doc":"parallel forward one"},
                {"connection_id":id(161),"from_port_id":port(70),"to_port_id":port(71),"rule_template_id":id(63),"tear_cost":3.0,"doc":"parallel forward two"},
                {"connection_id":id(162),"from_port_id":port(71),"to_port_id":port(70),"rule_template_id":id(63),"tear_cost":4.0,"doc":"reverse cycle edge"}
            ]);
        });
    });
    let model = fixture.commit(vec![document]).await;
    let report = fixture.run(model, "P5").await.unwrap();
    let stage = &report.stages.last().unwrap().snapshot;
    assert_eq!(rows(&fixture.registry, stage, "inferred.ports").len(), 2);
    assert_eq!(
        rows(&fixture.registry, stage, "inferred.port_state_targets").len(),
        2
    );
    assert_eq!(
        rows(&fixture.registry, stage, "inferred.topology_edges").len(),
        3
    );
    let tears = rows(&fixture.registry, stage, "inferred.tear_candidates");
    let chosen = tears
        .iter()
        .filter(|row| {
            get(&fixture.registry, "inferred.tear_candidates", row, "chosen") == &Cell::Bool(true)
        })
        .map(|row| {
            get(
                &fixture.registry,
                "inferred.tear_candidates",
                row,
                "connection_id",
            )
            .literal_spec()
        })
        .collect::<BTreeSet<_>>();
    assert!(!chosen.is_empty());
    assert_eq!(
        chosen.contains(&Cell::Id(id(160)).literal_spec()),
        chosen.contains(&Cell::Id(id(161)).literal_spec()),
        "parallel edges share actual directed group"
    );
    assert!(
        chosen.contains(&Cell::Id(id(162)).literal_spec())
            || chosen.contains(&Cell::Id(id(160)).literal_spec()),
        "removing selected edges breaks the actual two-node cycle"
    );
    assert_eq!(tears.len(), 3);
    assert_eq!(
        stage
            .relation("inferred", "boundary_crossings")
            .unwrap()
            .batch()
            .num_rows(),
        6
    );
}

#[tokio::test]
async fn separate_phase_species_axes_reject_only_actual_forbidden_pairs() {
    let mut fixture = native_pipeline::Fixture::new();
    let document = changed(&fixture.registry, |texts| {
        edit(texts, "materials/system.yaml", |value| {
            value["material_systems"][0]["phase_ids"] = json!([id(170), id(171)]);
            value["material_systems"][0]["species_ids"] = json!([id(172), id(173)]);
            value["phases"] = json!([
                {"id":id(170),"package_id":id(90),"name":"liquid","phase_type":"liquidPhase","is_solvent_phase":false,"doc":"actual phase"},
                {"id":id(171),"package_id":id(90),"name":"vapor","phase_type":"vaporPhase","is_solvent_phase":false,"doc":"actual phase"}
            ]);
            value["species"]=[172,173].into_iter().map(|member|json!({"id":id(member),"package_id":id(90),"name":format!("species{member}"),"formula":null,"mw":null,"component_type":"Component","charge":0,"dissociation_species":null,"valid_phase_types":["liquidPhase","vaporPhase"],"doc":"explicit phase type allowance"})).collect();
            value["phase_species"] = json!([{"phase_id":id(170),"species_id":id(172)},{"phase_id":id(171),"species_id":id(173)}]);
        });
        edit(texts, "templates/state.yaml", |value| {
            value["template_domains"]=[("p","phase"),("s","species")].into_iter().map(|(name,kind)|json!({"template_id":id(60),"name":name,"kind":kind,"continuous":false,"members_from":null,"bounds":null,"unit_id":null})).collect();
            value["template_domain_bindings"]=[("p","phase"),("s","species")].into_iter().map(|(name,source)|json!({"template_id":id(60),"name":name,"source":source,"domain_id":null,"parameter_name":null})).collect();
            value["template_equations"].as_array_mut().unwrap().push(json!({"id":id(174),"template_id":id(60),"name":"pair_shape","indexed_by":["p","s"],"expression":"1 == 1","sense":"eq","doc":"actual separate product factors"}));
        });
    });
    let model = fixture.commit(vec![document]).await;
    let report = fixture.run(model, "P5").await.unwrap();
    let p3 = &report
        .stages
        .iter()
        .find(|stage| stage.pass == "P3")
        .unwrap()
        .snapshot;
    let p4 = &report
        .stages
        .iter()
        .find(|stage| stage.pass == "P4")
        .unwrap()
        .snapshot;
    let p5 = &report.stages.last().unwrap().snapshot;
    let bindings = rows(&fixture.registry, p3, "normalized.instance_domain_bindings");
    let domain = |name| match get(
        &fixture.registry,
        "normalized.instance_domain_bindings",
        bindings
            .iter()
            .find(|row| {
                get(
                    &fixture.registry,
                    "normalized.instance_domain_bindings",
                    row,
                    "domain_name",
                ) == &Cell::text(name)
            })
            .unwrap(),
        "domain_id",
    ) {
        Cell::Id(id) => *id,
        _ => panic!("actual domain identity"),
    };
    let products = rows(&fixture.registry, p3, "normalized.domain_products");
    let expected_factors = Cell::List(vec![Cell::Id(domain("p")), Cell::Id(domain("s"))]);
    let product = match &products
        .iter()
        .find(|row| row[1] == expected_factors)
        .unwrap()[0]
    {
        Cell::Id(id) => *id,
        _ => panic!("declared product identity"),
    };
    let candidate = rows(&fixture.registry, p3, "normalized.candidate_index_tuples")
        .into_iter()
        .filter(|row| row[0] == Cell::Id(product))
        .collect::<Vec<_>>();
    let valid = rows(&fixture.registry, p5, "inferred.valid_index_tuples")
        .into_iter()
        .filter(|row| row[0] == Cell::Id(product))
        .collect::<Vec<_>>();
    assert_eq!(candidate.len(), 4);
    assert_eq!(
        valid.len(),
        2,
        "both per-axis members exist, but only explicit matched pairs are eligible"
    );
    assert_eq!(
        rows(&fixture.registry, p4, "inferred.phase_species").len(),
        2
    );
    let members = rows(&fixture.registry, p3, "normalized.material_domain_members");
    let mut actual_pairs = BTreeSet::new();
    for row in valid {
        let Cell::List(tuple) = &row[1] else {
            panic!("exact tuple")
        };
        let phase = members
            .iter()
            .find(|member| member[0] == Cell::Id(domain("p")) && member[1] == tuple[0])
            .unwrap();
        let species = members
            .iter()
            .find(|member| member[0] == Cell::Id(domain("s")) && member[1] == tuple[1])
            .unwrap();
        actual_pairs.insert((phase[3].literal_spec(), species[4].literal_spec()));
    }
    assert_eq!(
        actual_pairs,
        [(170, 172), (171, 173)]
            .map(|(phase, species)| (
                Cell::Id(id(phase)).literal_spec(),
                Cell::Id(id(species)).literal_spec()
            ))
            .into_iter()
            .collect()
    );
}

#[tokio::test]
async fn collection_paths_and_ports_retain_every_actual_child_coordinate() {
    let mut fixture = native_pipeline::Fixture::new();
    let document = changed(&fixture.registry, |texts| {
        edit(texts, "templates/state.yaml", |value| {
            value["template_domains"] = json!([{"template_id":id(60),"name":"stage","kind":"stage","continuous":false,"members_from":null,"bounds":null,"unit_id":null}]);
            value["template_domain_bindings"] = json!([{"template_id":id(60),"name":"stage","source":"domain","domain_id":id(180),"parameter_name":null}]);
            value["template_submodels"] = json!([{"template_id":id(60),"name":"child","child_template_id":id(61),"child_from_param":null,"multiplicity_domain":"stage","bindings":[],"guard_id":null}]);
            value["template_equations"].as_array_mut().unwrap().push(json!({"id":id(183),"template_id":id(60),"name":"path_read","indexed_by":["stage"],"expression":"child[stage].output == 1","sense":"eq","doc":"exact child coordinate path"}));
            value["template_port_members"] = json!([{"template_id":id(61),"ordinal":0,"symbol_decl_id":id(43),"symbol_group":"output"}]);
            value["template_ports"] = json!([{"template_id":id(60),"name":"collection","kind":"material","direction":"inlet","bound_to":"child","guard_id":null,"doc":"complete finite child collection"}]);
        });
        texts.insert("materials/stages.yaml".into(),json!({
            "domains":[{"id":id(180),"owner_entity_id":id(70),"kind":"stage","continuous":false,"unit_id":null,"parent_domain_id":null,"doc":"actual declared child domain"}],
            "domain_members":[
                {"domain_id":id(180),"id":id(181),"ordinal":0,"label":"low","coordinate":10.0,"ref_entity_id":null},
                {"domain_id":id(180),"id":id(182),"ordinal":1,"label":"high","coordinate":30.0,"ref_entity_id":null}
            ]
        }).to_string());
    });
    let model = fixture.commit(vec![document]).await;
    let report = fixture.run(model, "P5").await.unwrap();
    let stage = &report.stages.last().unwrap().snapshot;
    let paths = rows(&fixture.registry, stage, "inferred.path_targets");
    let targets = paths
        .iter()
        .filter(|row| {
            get(
                &fixture.registry,
                "inferred.path_targets",
                row,
                "symbol_decl_id",
            ) == &Cell::Id(id(43))
        })
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 2);
    assert_eq!(
        targets
            .iter()
            .map(|row| get(
                &fixture.registry,
                "inferred.path_targets",
                row,
                "path_index"
            )
            .literal_spec())
            .collect::<BTreeSet<_>>(),
        [181, 182]
            .map(|member| Cell::List(vec![Cell::Id(id(member))]).literal_spec())
            .into_iter()
            .collect()
    );
    for target in targets {
        assert_eq!(
            get(
                &fixture.registry,
                "inferred.path_targets",
                target,
                "path_domains"
            ),
            &Cell::List(vec![Cell::Id(id(180))])
        );
    }
    let ports = rows(&fixture.registry, stage, "inferred.ports");
    assert_eq!(ports.len(), 1);
    assert_eq!(
        get(
            &fixture.registry,
            "inferred.ports",
            &ports[0],
            "state_instance_id"
        ),
        &Cell::Null,
        "collection has no representative state"
    );
    assert_eq!(
        rows(&fixture.registry, stage, "inferred.port_state_targets").len(),
        2
    );
    assert_eq!(
        rows(&fixture.registry, stage, "inferred.port_state_domains")[0][1],
        Cell::List(vec![Cell::Id(id(180))])
    );
}
