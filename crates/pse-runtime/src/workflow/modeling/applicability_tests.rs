// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Evidence is projected through the registry and failures preserve typed authorization.
use super::results::{applicability_checks,ModelingCheck};
use pse_model::applicability::{Claim,Node,Permission,Region};
use pse_model::generated::enums::{ModelingApplicabilityOutcome as Outcome,ModelingPermissionTarget as Target,ModelingValidityLayer as Layer};
use pse_relations::RelationRow;
use pse_ids::SemanticId;
fn id(n:u8)->SemanticId {SemanticId::from_bytes([n;16])}
fn observed()->pse_model::applicability::Observation {
    let plan=Node {claim:Claim {id:None,coverage:Some(id(2)),owner:id(3),evidence:Some(id(4)),form:id(5),call:id(6),records:vec![id(7)],dependencies:vec![id(8)],layer:Layer::Data,basis:None,reason:Some("Published record lacks a scientific applicability region".into())},region:Region::Unknown,dependencies:vec![],inputs:vec![("T".into(),0,id(9))],permissions:vec![Permission {id:id(10),scope:id(11),target_kind:Target::Records,targets:vec![id(7)],allow_unknown:true,allow_extrapolation:false}]};
    let mut observation=plan.assess(&[],&[412.5]).observations.remove(0);
    observation.instance=Some(id(12));observation
}
#[test]
fn applicability_check_registry_roundtrip_preserves_physical_inputs_and_claim_lineage() {
    let observation=observed();let checks=applicability_checks(id(13).into(),std::slice::from_ref(&observation));
    let check=&checks[0];assert_eq!(check.applicability_outcome,Some(Outcome::UnknownEvidence));
    assert_eq!(check.claim_id,None);assert_eq!(check.claim_owner,Some(id(3)));assert_eq!(check.call_id,Some(id(6)));
    assert_eq!(check.coverage_id,Some(id(2)));assert_eq!(check.evidence_id,Some(id(4)));assert_eq!(check.form_id,Some(id(5)));
    assert_eq!(check.selected_records,vec![id(7)]);assert_eq!(check.dependencies,vec![id(8)]);assert_eq!(check.permission_ids,vec![id(10)]);
    assert_eq!(check.input_values[0].quantity_type,id(9));assert_eq!(check.input_values[0].name,"T");assert_eq!(check.input_values[0].value,412.5);
    assert!(check.satisfied);assert_eq!(check.unknown_allowed,Some(true));assert_eq!(check.extrapolation_allowed,Some(false));
    let registry=pse_schema::registry().unwrap();let mut builder=ModelingCheck::builder(registry,1).unwrap();
    ModelingCheck::push(&mut builder,check.clone()).unwrap();let batch=ModelingCheck::finish(builder).unwrap();
    assert_eq!(ModelingCheck::rows(&batch).unwrap(),checks);
}
#[test]
fn structured_refusal_serialization_preserves_instance_scope_and_independent_permissions() {
    let mut observation=observed();observation.permissions.clear();observation.unknown_allowed=false;observation.admitted=false;
    let mut diagnostic=pse_model::diagnostic::BoundaryDiagnostic::new(pse_model::diagnostic::BoundaryClass::Domain,"applicability",vec![id(5),id(7)],"Required empirical evidence is unknown");
    diagnostic.applicability.push(observation.clone());
    let encoded=serde_json::to_vec(&diagnostic).unwrap();let decoded: pse_model::diagnostic::BoundaryDiagnostic=serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded.applicability,vec![observation]);
    let permission=observed().permissions.remove(0);assert_eq!(permission.scope,id(11));assert_eq!(permission.targets,vec![id(7)]);
    assert!(permission.allow_unknown);assert!(!permission.allow_extrapolation);
}
