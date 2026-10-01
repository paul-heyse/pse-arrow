// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Current authored parameter contracts; no copied scientific functions or full bank journey.
use crate::specialize::Value;
use crate::{CheckedPackage, Declaration, PhysicalScope, Result, TypeContext, check};
use pse_ids::SemanticId;

const SOURCES: &[&str] = &[
    include_str!("../../../packages/reference/physical/models/chemistry.pse"),
    include_str!("../../../packages/reference/physical/models/compatibility.pse"),
    include_str!("../../../packages/reference/domain/models/provenance.pse"),
    include_str!("../../../packages/reference/domain/models/constants.pse"),
    include_str!("../../../packages/reference/domain/models/properties.pse"),
    include_str!("../../../packages/reference/domain/models/interactions.pse"),
    include_str!("../../../packages/reference/methods/models/nrtl.pse"),
    include_str!("../../../packages/reference/methods/models/cubic.pse"),
    include_str!("../../../packages/reference/methods/models/pcsaft-parameters.pse"),
    include_str!("../../../packages/reference/methods/models/correlations.pse"),
];

fn admitted(fixture: &str) -> Result<CheckedPackage> {
    let mut declarations: Vec<Declaration> = Vec::new();
    for source in SOURCES.iter().copied().chain(std::iter::once(fixture)) {
        declarations.extend(
            pse_authoring::language::parse(
                source,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap(),
        );
    }
    let quantities = pse_quantity::standard::standard_registry().unwrap();
    let preconditions =
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap();
    check(
        &declarations,
        &TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &quantities,
            preconditions: &preconditions,
            scope: &PhysicalScope::default(),
        },
    )
}
fn records<'a>(p: &'a CheckedPackage, context: &str) -> &'a [Value] {
    let Value::Set(records) = &p.entities[&p.names[context]].values["records"] else {
        panic!("retained closed record references")
    };
    records
}
fn failure(fixture: &str) -> String {
    match admitted(fixture) {
        Ok(_) => panic!("expected selection refusal"),
        Err(error) => error.to_string(),
    }
}

fn refuses(fixture: &str, expected: &str) {
    let error = failure(fixture);
    assert!(error.contains(expected), "expected {expected}: {error}");
}

const BASE: &str = r#"package selection_fixture {
use chemistry @"1.0.0"; use compatibility @"1.0.0"; use properties @"1.0.0";
use interactions @"1.0.0"; use provenance @"1.0.0"; use nrtl @"1.0.0";
@id("10000000000000000000000000000001") entity chemistry.species a {}
@id("10000000000000000000000000000002") entity chemistry.species b {}
@id("10000000000000000000000000000003") entity chemistry.species c {}
@id("10000000000000000000000000000004") entity chemistry.species d {}
entity provenance.source publication {title="One publication, independently identifiable fits"}
entity properties.parameterization fit_x {title="Independent A-B fit X",source=publication}
entity properties.parameterization fit_y {title="Independent A-C and B-C fits Y",source=publication}
entity properties.property pair_contract {quantity=Scalar,shape=properties.IndexShape.pair,applies={compatibility.PhaseType.liquidPhase,compatibility.PhaseType.vaporPhase}}
set packages:Set<properties.property_package>={model};
set families:Set<properties.property>={pair_contract};
set members:Set<chemistry.species>={a,b,c};
dataset x:interactions.symmetric_scalar bind(parameterization=fit_x,family=pair_contract,source=publication,dependencies={},conventions={}) provenance(publication,provenance.Role.published) {
 [a,b,1]=[0.1]; [a,b,2]=[0.7];
}
dataset y:interactions.symmetric_scalar bind(parameterization=fit_y,family=pair_contract,source=publication,dependencies={},conventions={}) provenance(publication,provenance.Role.published) {
 [a,c]=[0.2]; [b,c]=[0.3];
}
dataset choices:interactions.symmetric_selection complete_over(k in packages,family in families,i in members,j in members) provenance(publication,provenance.Role.published) {
 [model,pair_contract,a,b]=[interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],missing];
 [model,pair_contract,a,c]=[interactions.symmetric_scalar[fit_y,pair_contract,a,c],missing];
 [model,pair_contract,b,c]=[interactions.symmetric_scalar[fit_y,pair_contract,b,c],missing];
}
entity properties.selection_context selected {
 roots={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],interactions.symmetric_scalar[fit_y,pair_contract,a,c],interactions.symmetric_scalar[fit_y,pair_contract,b,c]},
 subjects=members,models=families,rules={}
}
entity properties.property_package model {admitted=selected}
}"#;

#[test]
fn authored_ternary_selects_existing_independent_records_without_coefficient_copies() {
    let p = admitted(BASE).unwrap();
    let closed = records(&p, "selection_fixture.selected");
    assert_eq!(closed.len(), 3);
    let selected = closed
        .iter()
        .map(|value| {
            let Value::Entity { id, .. } = value else {
                panic!("record")
            };
            &p.entities[id]
        })
        .collect::<Vec<_>>();
    assert_eq!(
        selected
            .iter()
            .filter(|r| r.values["parameterization"]
                == Value::Entity {
                    id: p.names["selection_fixture.fit_x"],
                    kind: p.names["properties.parameterization"]
                })
            .count(),
        1
    );
    assert!(selected.iter().all(|r| r.values["source"]
        == Value::Entity {
            id: p.names["selection_fixture.publication"],
            kind: p.names["provenance.source"]
        }));
    assert_eq!(
        p.entities
            .values()
            .filter(|r| r.kind == p.names["interactions.symmetric_scalar"])
            .count(),
        4
    );
    assert!(
        !p.selection_closures().is_empty(),
        "admitted selection retains its closure, not only numbers"
    );
}

#[test]
fn same_publication_variants_are_distinct_but_conflicting_slot_selection_refuses() {
    let fixture=BASE.replace(
        "roots={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],",
        "roots={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],interactions.symmetric_scalar[fit_x,pair_contract,a,b,2],",
    );
    refuses(&fixture, "conflicting fits or variants");
    let fixture=BASE.replace("[a,c]=[0.2];", "[a,c]=[0.2]; [a,b]=[0.9];")
        .replace("roots={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],",
                 "roots={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],interactions.symmetric_scalar[fit_y,pair_contract,a,b],");
    assert!(
        failure(&fixture).contains("conflicting fits or variants"),
        "same variant number in different fits does not authorize interchangeability"
    );
}

#[test]
fn missing_required_pair_is_not_an_implicit_zero() {
    let fixture=BASE.replace(" [model,pair_contract,b,c]=[interactions.symmetric_scalar[fit_y,pair_contract,b,c],missing];", "");
    refuses(&fixture, "missing");
}

#[test]
fn symmetric_reverse_record_and_double_orientation_refuse() {
    let fixture = BASE
        .replace("[a,b,1]=[0.1]", "[b,a,1]=[0.1]")
        .replace("[fit_x,pair_contract,a,b,1]", "[fit_x,pair_contract,b,a,1]");
    refuses(&fixture, "canonical orientation");
    let fixture=BASE.replace(" [model,pair_contract,a,b]=[interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],missing];",
        " [model,pair_contract,a,b]=[interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],missing];\n [model,pair_contract,b,a]=[interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],missing];");
    refuses(&fixture, "both orientations");
}

#[test]
fn fitted_zero_and_explicit_prediction_keep_different_authority() {
    let fixture=BASE.replace("[a,b,1]=[0.1]", "[a,b,1]=[0]")
        .replace("[model,pair_contract,b,c]=[interactions.symmetric_scalar[fit_y,pair_contract,b,c],missing]",
                 "[model,pair_contract,b,c]=[missing,zero_prediction]")
        .replace(",interactions.symmetric_scalar[fit_y,pair_contract,b,c]},", "},")
        .replace("subjects=members,models=families,rules={}","subjects=members,models=families,rules={zero_prediction}")
        .replacen("entity properties.property_package model", "entity properties.predictive_rule zero_prediction {source=publication,family=pair_contract,dependencies={interactions.symmetric_scalar[fit_y,pair_contract,b,c]},output=interactions.predictive_zero}\nentity properties.property_package model",1);
    let p = admitted(&fixture).unwrap();
    let Value::Set(rules) = &p.entities[&p.names["selection_fixture.selected"]].values["rules"]
    else {
        panic!("explicit rules")
    };
    assert_eq!(rules.len(), 1);
    assert!(records(&p, "selection_fixture.selected").iter().any(|r| {
        let Value::Entity { id, .. } = r else {
            return false;
        };
        p.entities[id].values["value"].scalar(*id).unwrap() == 0.0
    }));
}

#[test]
fn explicit_dependencies_close_and_dangling_references_refuse() {
    let fixture=BASE.replace("parameterization=fit_x,family=pair_contract,source=publication,dependencies={}",
        "parameterization=fit_x,family=pair_contract,source=publication,dependencies={interactions.symmetric_scalar[fit_y,pair_contract,a,c]}")
        .replace("roots={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],interactions.symmetric_scalar[fit_y,pair_contract,a,c],interactions.symmetric_scalar[fit_y,pair_contract,b,c]}",
                 "roots={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],interactions.symmetric_scalar[fit_y,pair_contract,b,c]}");
    assert_eq!(
        records(&admitted(&fixture).unwrap(), "selection_fixture.selected").len(),
        3
    );
    let dangling = fixture.replace(
        "dependencies={interactions.symmetric_scalar[fit_y,pair_contract,a,c]}",
        "dependencies={interactions.symmetric_scalar[fit_y,pair_contract,a,d]}",
    );
    assert!(admitted(&dangling).is_err());
}

#[test]
fn joint_fit_needs_a_declared_subsystem_projection() {
    let fixture=BASE.replace("[a,c]=[0.2]; [b,c]=[0.3];", "[a,c]=[0.2]; [b,c]=[0.3]; [c,d]=[0.4];")
        .replacen("entity properties.property_package model", "entity properties.fit_group joint {members={interactions.symmetric_scalar[fit_y,pair_contract,a,c],interactions.symmetric_scalar[fit_y,pair_contract,b,c],interactions.symmetric_scalar[fit_y,pair_contract,c,d]}}\nentity properties.property_package model",1);
    refuses(&fixture, "declared projection");
    let projected=fixture.replacen("entity properties.property_package model", r#"entity properties.permitted_projection abc {group=joint,subjects=members,records={interactions.symmetric_scalar[fit_y,pair_contract,a,c],interactions.symmetric_scalar[fit_y,pair_contract,b,c]}}
 dataset project:properties.selected_projection provenance(publication,provenance.Role.published) {[selected,joint]=[abc];}
 entity properties.property_package model"#,1);
    assert_eq!(
        records(&admitted(&projected).unwrap(), "selection_fixture.selected").len(),
        3
    );
}

#[test]
fn nrtl_reverse_selection_cannot_relabel_an_ordered_fit() {
    let fixture=BASE.replacen("entity properties.property_package model", r#"dataset directed:nrtl.parameters bind(parameterization=fit_x,family=nrtl.interaction_parameters,source=publication,dependencies={},conventions={}) provenance(publication,provenance.Role.published) {[a,b]=[1,0.3]; [b,a]=[2,0.3];}
 entity properties.selection_context ordered {roots={nrtl.parameters[fit_x,nrtl.interaction_parameters,a,b],nrtl.parameters[fit_x,nrtl.interaction_parameters,b,a]},subjects=members,models={nrtl.interaction_parameters},rules={}}
 entity properties.property_package nrtl_model {admitted=ordered}
 set ordered_members:Set<chemistry.species>={a,b};
 set ordered_models:Set<properties.property_package>={nrtl_model};
  dataset chosen:nrtl.selection complete_over(k in ordered_models,i in ordered_members,j in ordered_members) provenance(publication,provenance.Role.published) {[nrtl_model,a,b]=[nrtl.parameters[fit_x,nrtl.interaction_parameters,a,b]]; [nrtl_model,b,a]=[nrtl.parameters[fit_x,nrtl.interaction_parameters,b,a]];}
 entity properties.property_package model"#,1);
    admitted(&fixture).unwrap();
    let wrong = fixture.replace(
        "[nrtl_model,b,a]=[nrtl.parameters[fit_x,nrtl.interaction_parameters,b,a]]",
        "[nrtl_model,b,a]=[nrtl.parameters[fit_x,nrtl.interaction_parameters,a,b]]",
    );
    assert!(admitted(&wrong).is_err());
}

#[test]
fn atomic_membership_is_closed_without_an_authored_backlink() {
    let fixture=BASE.replace("roots={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],interactions.symmetric_scalar[fit_y,pair_contract,a,c],interactions.symmetric_scalar[fit_y,pair_contract,b,c]}",
        "roots={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],interactions.symmetric_scalar[fit_y,pair_contract,b,c]}")
        .replacen("entity properties.property_package model", "entity properties.fit_group joint {members={interactions.symmetric_scalar[fit_x,pair_contract,a,b,1],interactions.symmetric_scalar[fit_y,pair_contract,a,c]}}\nentity properties.property_package model",1);
    let p = admitted(&fixture).unwrap();
    assert_eq!(records(&p, "selection_fixture.selected").len(), 3);
}


#[test]
fn optional_record_unwrap_refuses_absence_and_preserves_the_supplied_identity() {
    let fixture=BASE.replacen("entity properties.property_package model",r#"entity kind index_probe {
 derived chosen:Set<interactions.symmetric_scalar>=set_of(require_present(interactions.symmetric_records[fit_x,pair_contract,a,b,1]));
}
entity index_probe demanded {}
entity properties.property_package model"#,1);
    refuses(&fixture,"required optional value is absent");
    let supplied=fixture.replacen("entity properties.property_package model",r#"dataset indexed:interactions.symmetric_records provenance(publication,provenance.Role.published) {
 [fit_x,pair_contract,a,b,1]=[interactions.symmetric_scalar[fit_x,pair_contract,a,b,1]];
}
entity properties.property_package model"#,1);
    let p=admitted(&supplied).unwrap();
    let Value::Set(chosen)=&p.entities[&p.names["selection_fixture.demanded"]].values["chosen"] else {panic!("chosen existing record")};
    assert_eq!(chosen.len(),1);
    assert!(records(&p,"selection_fixture.selected").contains(&chosen[0]));
}
