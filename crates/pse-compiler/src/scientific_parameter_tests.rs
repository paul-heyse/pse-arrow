// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected production scientific bodies with small independently specified parameter records.
#![allow(clippy::unwrap_used, reason = "focused scientific selection controls")]
use crate::workspace::{CompilerWorkspace, Profile, WorkspaceLimits};
use pse_ids::SemanticId;
use pse_math::binding::CaseValues;
use pse_modeling::{Bindings, Limits, PhysicalScope, specialize::root_instance};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};

const SOURCES: &[&str] = &[
    include_str!("../../../packages/reference/physical/models/chemistry.pse"),
    include_str!("../../../packages/reference/physical/models/compatibility.pse"),
    include_str!("../../../packages/reference/domain/models/constants.pse"),
    include_str!("../../../packages/reference/domain/models/provenance.pse"),
    include_str!("../../../packages/reference/domain/models/properties.pse"),
    include_str!("../../../packages/reference/domain/models/interactions.pse"),
    include_str!("../../../packages/reference/methods/models/nrtl.pse"),
    include_str!("../../../packages/reference/methods/models/cubic.pse"),
];
fn check(name: &str) {
    let source = format!("{}\n{FIXTURE}", SOURCES.join("\n"));
    let rows = pse_authoring::language::parse(
        &source,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let package = rows
        .iter()
        .find(|r| r.parent_id.is_none() && r.name == "parameter_point")
        .unwrap()
        .declaration_id;
    let root = rows
        .iter()
        .find(|r| r.parent_id == Some(package) && r.name == name)
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::inputs(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    let point = workspace
        .check_modeling_point(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(!point.expectations.is_empty());
    assert!(
        point.expectations.iter().all(|e| e.passed),
        "{name}: {point:?}"
    );
}
#[test]
fn ternary_nrtl_uses_independent_ordered_record_fits_in_the_actual_potential() {
    check("nrtl_point");
}
#[test]
fn selected_symmetric_fitted_zero_prediction_and_directional_values_remain_distinct() {
    check("pair_point");
}

const FIXTURE: &str = r#"package parameter_point {
use chemistry @"1.0.0";use compatibility @"1.0.0";use properties @"1.0.0";
use provenance @"1.0.0";use interactions @"1.0.0";use nrtl @"1.0.0";use cubic @"1.0.0";
@id("30000000000000000000000000000001") entity chemistry.species a {}
@id("30000000000000000000000000000002") entity chemistry.species b {}
@id("30000000000000000000000000000003") entity chemistry.species c {}
entity provenance.source evidence {title="Independent synthetic numerical controls; no scientific coefficient bank is copied"}
entity properties.parameterization fit_x {title="A-B ordered fit",source=evidence}
entity properties.parameterization fit_y {title="A-C and B-C ordered fits",source=evidence}
set members:Set<chemistry.species>={a,b,c};
set packages:Set<properties.property_package>={model};
dataset x:nrtl.parameters bind(parameterization=fit_x,family=nrtl.interaction_parameters,source=evidence,dependencies={},conventions={}) provenance(evidence,provenance.Role.synthetic) {[a,b]=[0.8,0.3];[b,a]=[-0.2,0.3];}
dataset y:nrtl.parameters bind(parameterization=fit_y,family=nrtl.interaction_parameters,source=evidence,dependencies={},conventions={}) provenance(evidence,provenance.Role.synthetic) {[a,c]=[1.1,0.3];[c,a]=[0.4,0.3];[b,c]=[-0.3,0.3];[c,b]=[0.6,0.3];}
entity properties.selection_context selected {roots={nrtl.parameters[fit_x,nrtl.interaction_parameters,a,b],nrtl.parameters[fit_x,nrtl.interaction_parameters,b,a],nrtl.parameters[fit_y,nrtl.interaction_parameters,a,c],nrtl.parameters[fit_y,nrtl.interaction_parameters,c,a],nrtl.parameters[fit_y,nrtl.interaction_parameters,b,c],nrtl.parameters[fit_y,nrtl.interaction_parameters,c,b]},subjects=members,models={nrtl.interaction_parameters},rules={}}
entity properties.property_package model {admitted=selected}
dataset chosen:nrtl.selection complete_over(k in packages,i in members,j in members) provenance(evidence,provenance.Role.synthetic) {
[model,a,b]=[nrtl.parameters[fit_x,nrtl.interaction_parameters,a,b]];
[model,b,a]=[nrtl.parameters[fit_x,nrtl.interaction_parameters,b,a]];
[model,a,c]=[nrtl.parameters[fit_y,nrtl.interaction_parameters,a,c]];
[model,c,a]=[nrtl.parameters[fit_y,nrtl.interaction_parameters,c,a]];
[model,b,c]=[nrtl.parameters[fit_y,nrtl.interaction_parameters,b,c]];
[model,c,b]=[nrtl.parameters[fit_y,nrtl.interaction_parameters,c,b]];
}
// Frozen from the independent extensive sum and its analytical amount derivative at x=(.2,.3,.5).
test nrtl_point fixture {dof 0;run pure;} {
permission selected_fit families(nrtl.parameters) allow_unknown true allow_extrapolation false;
let n[j in members]:Amount=if j==a then 0.2{mol} else if j==b then 0.3{mol} else 0.5{mol};
expect nrtl.normalized_excess(300{K},101325{Pa},n,members,model,nrtl.potential)==0.1834063116391329 tolerance 1e-12;
expect nrtl.ln_gamma(300{K},101325{Pa},n,members,model,a)==0.5774019102100152{1} tolerance 1e-12{1};
expect nrtl.ln_gamma(300{K},101325{Pa},n,members,model,b)==0.00907812037155692{1} tolerance 1e-12{1};
expect nrtl.ln_gamma(300{K},101325{Pa},n,members,model,c)==0.13040498697132558{1} tolerance 1e-12{1};
}
entity properties.property scalar_pair {quantity=Scalar,shape=properties.IndexShape.pair,applies={compatibility.PhaseType.liquidPhase}}
dataset scalars:interactions.symmetric_scalar bind(parameterization=fit_x,family=scalar_pair,source=evidence,dependencies={},conventions={}) provenance(evidence,provenance.Role.synthetic) {[a,b]=[0.1];[a,c]=[0];[b,c]=[0.3];}
entity properties.predictive_rule predicted {source=evidence,family=scalar_pair,dependencies={interactions.symmetric_scalar[fit_x,scalar_pair,b,c]},output=interactions.predictive_zero}
entity properties.selection_context scalar_selection {roots={interactions.symmetric_scalar[fit_x,scalar_pair,a,b],interactions.symmetric_scalar[fit_x,scalar_pair,a,c]},subjects=members,models={scalar_pair},rules={predicted}}
entity properties.property_package scalar_model {admitted=scalar_selection}
set scalar_packages:Set<properties.property_package>={scalar_model};set scalar_families:Set<properties.property>={scalar_pair};
dataset scalar_choices:interactions.symmetric_selection complete_over(k in scalar_packages,family in scalar_families,i in members,j in members) provenance(evidence,provenance.Role.synthetic) {
[scalar_model,scalar_pair,a,b]=[interactions.symmetric_scalar[fit_x,scalar_pair,a,b],missing];
[scalar_model,scalar_pair,a,c]=[interactions.symmetric_scalar[fit_x,scalar_pair,a,c],missing];
[scalar_model,scalar_pair,b,c]=[missing,predicted];
}
dataset self_choice:interactions.self_rule complete_over(k in scalar_packages,family in scalar_families) provenance(evidence,provenance.Role.synthetic) {[scalar_model,scalar_pair]=[interactions.zero_self];}
dataset directed:cubic.directional_pair bind(parameterization=fit_x,family=cubic.binary_interaction,source=evidence,dependencies={},conventions={}) provenance(evidence,provenance.Role.synthetic) {[a,b]=[0.1];[b,a]=[0.3];}
entity properties.selection_context ordered_selection {roots={cubic.directional_pair[fit_x,cubic.binary_interaction,a,b],cubic.directional_pair[fit_x,cubic.binary_interaction,b,a]},subjects={a,b},models={cubic.binary_interaction},rules={}}
entity properties.property_package ordered_model {admitted=ordered_selection}
set ordered_members:Set<chemistry.species>={a,b};set ordered_packages:Set<properties.property_package>={ordered_model};set ordered_families:Set<properties.property>={cubic.binary_interaction};dataset ordered_choices:cubic.directional_selection complete_over(k in ordered_packages,family in ordered_families,i in ordered_members,j in ordered_members) provenance(evidence,provenance.Role.synthetic) {[ordered_model,cubic.binary_interaction,a,b]=[cubic.directional_pair[fit_x,cubic.binary_interaction,a,b]];[ordered_model,cubic.binary_interaction,b,a]=[cubic.directional_pair[fit_x,cubic.binary_interaction,b,a]];}
test pair_point fixture {dof 0;run pure;} {
permission prediction_use families(properties.predictive_rule,interactions.symmetric_scalar,cubic.directional_pair) allow_unknown true allow_extrapolation false;
expect interactions.pair_parameter(scalar_model,scalar_pair,a,b)==0.1 tolerance 1e-15;
expect interactions.pair_parameter(scalar_model,scalar_pair,b,a)==0.1 tolerance 1e-15;
expect interactions.pair_parameter(scalar_model,scalar_pair,a,c)==0 tolerance 1e-15;
expect interactions.pair_parameter(scalar_model,scalar_pair,b,c)==0 tolerance 1e-15;
expect interactions.pair_parameter(scalar_model,scalar_pair,a,a)==0 tolerance 1e-15;
expect cubic.directional_parameter(ordered_model,cubic.binary_interaction,a,b)==0.1 tolerance 1e-15;
expect cubic.directional_parameter(ordered_model,cubic.binary_interaction,b,a)==0.3 tolerance 1e-15;
}
}"#;
