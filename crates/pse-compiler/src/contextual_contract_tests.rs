// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Focused physical-context execution through the ordinary mathematical compiler.
use crate::workspace::{CompilerWorkspace, Inputs, Profile, WorkspaceLimits};
use pse_authoring::{ParseBudget,language::{parse,IdentityPolicy}};
use pse_ids::SemanticId;
use pse_math::binding::CaseValues;
use pse_modeling::{Bindings,Limits,PhysicalScope,specialize::root_instance};
use std::{collections::BTreeMap,sync::{Arc,atomic::AtomicBool}};

fn workspace(text:&str)->(CompilerWorkspace,Vec<pse_modeling::Declaration>) {
    let rows=parse(text,SemanticId::from_bytes([192;16]),IdentityPolicy::Named,ParseBudget::default()).unwrap();
    let inputs=Inputs {quantities:Arc::new(pse_quantity::standard::standard_registry().unwrap()),preconditions:Arc::new(pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap()),flows:BTreeMap::new(),definitions:BTreeMap::new(),domains:BTreeMap::new(),groups:BTreeMap::new(),providers:BTreeMap::new(),cases:BTreeMap::new(),values:BTreeMap::new()};
    let mut workspace=CompilerWorkspace::new(inputs,WorkspaceLimits::default()).unwrap();
    workspace.publish_modeling(rows.clone(),PhysicalScope::default()).unwrap();
    (workspace,rows)
}
fn check(text:&str,name:&str) {
    let (mut workspace,rows)=workspace(text);
    let root=rows.iter().find(|row|row.name==name && row.value.kind==pse_model::generated::enums::ModelingDeclarationKind::Test).unwrap().declaration_id;
    let result=workspace.check_modeling_point(root,root_instance(root),Bindings::default(),Limits::default(),&CaseValues {scalars:BTreeMap::new()},Profile::default(),Arc::new(AtomicBool::new(false))).unwrap();
    assert!(!result.expectations.is_empty());
    assert!(result.expectations.iter().all(|value|value.passed),"{name}: {result:?}");
}

#[test]
fn transfer_conventions_and_consumption_execute_as_ordinary_functions() {
    check(r#"package p {
      test transfer_values {
        boundary surface;
        let rate:EnergyTransferRate=-25{W};
        let entering:Transfer<EnergyTransferRate,surface,Into>=transfer(rate,surface,Into);
        let leaving:Transfer<EnergyTransferRate,surface,OutOf>=reorient(entering,OutOf);
        accumulate ledger:Power boundary surface accounting tolerance 1e-9{W};
        contribute ledger role directed=entering;
        expect entering==-25{W} tolerance 1e-9{W};
        expect leaving==25{W} tolerance 1e-9{W};
        expect ledger==-25{W} tolerance 1e-9{W};
      }
    }"#,"transfer_values");
}

#[test]
fn reference_translation_keeps_composition_and_inverse_in_library_expression() {
    check(REFERENCE,"translated_values");
}

const REFERENCE:&str=r#"package p {
 @id("1c998211e5d74955863f377662f56526") entity kind species {}
 entity species a {} entity species b {}
 set members:Set<species>={a,b};
 entity kind source provenance {} entity source book {}
 enum Role { published }
 fn first_anchor(T:Temperature,p:Pressure,j:species)->BtOracleEnthalpy valid(T==300{K} and p==100000{Pa})=if j==a then 100{J/mol} else 200{J/mol};
 fn second_anchor(T:Temperature,p:Pressure,j:species)->MolarEnthalpy valid(T==300{K} and p==100000{Pa})=if j==a then 10{J/mol} else 20{J/mol};
 reference translation forward(value:BtOracleEnthalpy,composition:MoleFraction[species],actual:Set<species>)->MolarEnthalpy anchors(source=first_anchor,target=second_anchor) at(temperature=300{K},pressure=100000{Pa}) provenance(book,Role.published);
 reference translation inverse(value:MolarEnthalpy,composition:MoleFraction[species],actual:Set<species>)->BtOracleEnthalpy anchors(source=second_anchor,target=first_anchor) at(temperature=300{K},pressure=100000{Pa}) provenance(book,Role.published);
 test translated_values {
   let weights[j in members]:MoleFraction=if j==a then 0.25{1} else 0.75{1};
   let other[j in members]:MoleFraction=if j==a then 0.75{1} else 0.25{1};
   let input:BtOracleEnthalpy=250{J/mol};
   let translated:MolarEnthalpy=forward(input,weights,members);
   expect translated==92.5{J/mol} tolerance 1e-10{J/mol};
   expect forward(input,other,members)==137.5{J/mol} tolerance 1e-10{J/mol};
   expect inverse(translated,weights,members)==250{J/mol} tolerance 1e-10{J/mol};
 }
}"#;
