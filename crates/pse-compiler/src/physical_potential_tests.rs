// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated authored-potential admission and library evaluation, without storage or solvers.
#![allow(
    clippy::unwrap_used,
    reason = "focused physical-potential unit assertions"
)]

use crate::workspace::{CompilerContext, CompilerWorkspace, Profile, WorkspaceLimits};
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};
use pse_ids::SemanticId;
use pse_math::binding::CaseValues;
use pse_modeling::{Bindings, Limits, PhysicalScope, specialize::root_instance};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};

/// Whole current source documents: the tests do not copy scientific function bodies.
const SOURCES: &[&str] = &[
    include_str!("../../../packages/reference/physical/models/kinds.pse"),
    include_str!("../../../packages/reference/physical/models/compatibility.pse"),
    include_str!("../../../packages/reference/physical/models/chemistry.pse"),
    include_str!("../../../packages/reference/domain/models/provenance.pse"),
    include_str!("../../../packages/reference/domain/models/constants.pse"),
    include_str!("../../../packages/reference/domain/models/properties.pse"),
    include_str!("../../../packages/reference/domain/models/interactions.pse"),
    include_str!("../../../packages/reference/methods/models/caloric.pse"),
    include_str!("../../../packages/reference/methods/models/pure-properties.pse"),
    include_str!("../../../packages/reference/methods/models/cubic.pse"),
    include_str!("../../../packages/reference/methods/models/pcsaft-parameters.pse"),
    include_str!("../../../packages/reference/methods/models/nrtl.pse"),
    include_str!("../../../packages/reference/data/references/models/references.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/helmholtz.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/peng-robinson.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/pcsaft.pse"),
    include_str!("../../../packages/reference/seed-data/models/pr-oracle.pse"),
];

fn workspace() -> (CompilerWorkspace, Vec<pse_modeling::Declaration>) {
    let text = format!("{}\n{FIXTURE}", SOURCES.join("\n"));
    let rows = parse(
        &text,
        SemanticId::from_bytes([193; 16]),
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .unwrap();
    let input = CompilerContext {
        quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
        preconditions: Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        ),

        providers: BTreeMap::new(),
    };
    let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(rows.clone(), PhysicalScope::default())
        .unwrap();
    (workspace, rows)
}

fn check(name: &str) {
    let (mut workspace, rows) = workspace();
    let root = rows
        .iter()
        .find(|row| {
            row.name == name
                && row.value.kind == pse_model::generated::enums::ModelingDeclarationKind::Test
        })
        .unwrap()
        .declaration_id;
    let result = workspace
        .check_modeling_point(
            root,
            root_instance(root),
            Bindings::default(),
            Limits {
                // This full PC-SAFT response includes authored physical partials and
                // their independent perturbation controls. Keep its construction
                // explicitly finite without changing the default production budget.
                body_occurrences: (name == "pcsaft_physical_partials").then_some(65536),
                ..Limits::default()
            },
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    assert!(!result.expectations.is_empty());
    assert!(
        result
            .expectations
            .iter()
            .all(|expectation| expectation.passed),
        "{name}: {result:?}"
    );
}

#[test]
fn reference_physical_potential_sources_admit() {
    let _ = workspace();
}

#[test]
fn reference_ideal_potential_retains_ideal_terms_and_zero_responses() {
    check("ideal_physical_responses");
}

#[test]
fn reference_nrtl_reconstruction_is_degree_one_and_activity_is_amount_invariant() {
    check("nrtl_physical_homogeneity");
}

#[test]
fn reference_cubic_fugacity_uses_fixed_volume_amount_partials() {
    // Independent frozen ordered-row and symmetric-derivative values already owned by the package.
    check("asymmetric_conventions");
}

#[test]
fn reference_pcsaft_physical_partials_match_independent_coordinate_perturbations() {
    check("pcsaft_physical_partials");
}

#[test]
fn mapped_composition_references_preserve_direct_reciprocal_values_and_physical_partials() {
    check("composition_reference_partials");
}

#[test]
fn mapped_cubic_polynomials_preserve_independent_frozen_values() {
    check("cubic_polynomial_frontdoors");
}

const FIXTURE: &str = r#"
package physical_potential_fixture {
use peng_robinson @"1.0.0"; use cubic @"1.0.0"; use pr_oracle @"1.0.0";
use chemistry @"1.0.0";
use constants @"1.0.0";
use provenance @"1.0.0";
use properties @"1.0.0";
use interactions @"1.0.0";
use compatibility @"1.0.0";
use helmholtz @"1.0.0";
use pcsaft @"1.0.0";
use pcsaft_parameters @"1.0.0";
use nrtl @"1.0.0";
entity chemistry.species a {}
entity chemistry.species b {}
set members:Set<chemistry.species>={a,b};
entity provenance.source data_source {title="Focused two-component potential control"}
entity properties.parameterization fit {title="Named synthetic potential-control fit",source=data_source}
entity properties.predictive_rule zero_prediction {source=data_source,family=pcsaft_parameters.binary_interaction,output=interactions.predictive_zero}
entity properties.property_package parameters {admitted=selected}
set packages:Set<properties.property_package>={parameters};
set saft_pairs:Set<properties.property>={pcsaft_parameters.binary_interaction};
set saft_properties:Set<properties.property>={pcsaft_parameters.segments};
dataset nrtl_parameters:nrtl.parameters bind(parameterization=fit,family=nrtl.interaction_parameters,source=data_source,dependencies={},conventions={}) provenance(data_source,provenance.Role.synthetic) {
[a,b]=[1,0.3]; [b,a]=[1,0.3];
}
dataset selected_nrtl:nrtl.selection complete_over(k in packages,i in members,j in members) provenance(data_source,provenance.Role.synthetic) {
[parameters,a,b]=[nrtl.parameters[fit,nrtl.interaction_parameters,a,b]];
[parameters,b,a]=[nrtl.parameters[fit,nrtl.interaction_parameters,b,a]];
}
dataset predicted_pairs:interactions.symmetric_selection complete_over(k in packages,family in saft_pairs,i in members,j in members) provenance(data_source,provenance.Role.synthetic) {
[parameters,pcsaft_parameters.binary_interaction,a,b]=[missing,zero_prediction];
}
dataset saft_self:interactions.self_rule complete_over(k in packages,family in saft_pairs) provenance(data_source,provenance.Role.synthetic) {[parameters,pcsaft_parameters.binary_interaction]=[interactions.zero_self];}
// Published Gross-Sadowski methane and ethane parameters; no Arrow/storage journey is needed.
dataset segments:pcsaft_parameters.nonassociating bind(parameterization=fit,property=pcsaft_parameters.segments,source=data_source,dependencies={},conventions={}) provenance(data_source,provenance.Role.synthetic) {
[a]=[1,3.7039e-10{m},150.03{K}]; [b]=[1.6069,3.5206e-10{m},191.42{K}];
}
dataset selection:properties.pure_selection complete_over(k in packages,j in members,p in saft_properties) provenance(data_source,provenance.Role.synthetic) {
[parameters,a,pcsaft_parameters.segments]=[pcsaft_parameters.nonassociating[a,pcsaft_parameters.segments,fit]];
[parameters,b,pcsaft_parameters.segments]=[pcsaft_parameters.nonassociating[b,pcsaft_parameters.segments,fit]];
}
entity properties.selection_context selected {roots={nrtl.parameters[fit,nrtl.interaction_parameters,a,b],nrtl.parameters[fit,nrtl.interaction_parameters,b,a],pcsaft_parameters.nonassociating[a,pcsaft_parameters.segments,fit],pcsaft_parameters.nonassociating[b,pcsaft_parameters.segments,fit]},subjects=members,models={nrtl.interaction_parameters,pcsaft_parameters.segments,pcsaft_parameters.binary_interaction},rules={zero_prediction}}
fn saft(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy=pcsaft.potential(T,V,n,members,parameters);
response pressure_difference from law(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>,dv:Volume,law:Fn(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy)->Pressure=sum(j in members | n[j])*constants.gas_constant*(T-constants.absolute_zero)/V-(law(T,V+dv,n,members)-law(T,V-dv,n,members))/(2*dv);
response cv_difference from law(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>,dt:DeltaTemperature,law:Fn(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy)->MolarCp=-(T-constants.absolute_zero)*(law(T+dt,V,n,members)-2*law(T,V,n,members)+law(T-dt,V,n,members))/(dt^2*sum(j in members | n[j]));
// Two coordinate directions for A=R*N*S^2/(T*V*D), S=300*n_a+600*n_b,
// D=2*n_a+n_b. Both references depend on composition; the frozen controls below
// follow independent derivatives of this closed physical expression.
coordinate map forward_reference(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>) {
slot tau=(sum(j in members | n[j]*(if j==a then 300 else 600))/sum(j in members | n[j]))*constants.temperature_scale/(T-constants.absolute_zero);
slot delta=(sum(j in members | n[j])/V)/(constants.density_unit*(1+(n[a]/constants.amount_unit)/(sum(j in members | n[j])/constants.molar_reference_amount)));
}
coordinate map reciprocal_reference(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>) {
slot theta=(T-constants.absolute_zero)/((sum(j in members | n[j]*(if j==a then 300 else 600))/sum(j in members | n[j]))*constants.temperature_scale);
slot delta=(sum(j in members | n[j])/V)/(constants.density_unit*(1+(n[a]/constants.amount_unit)/(sum(j in members | n[j])/constants.molar_reference_amount)));
}
reconstruction forward_family for forward_reference(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy reference data_source=sum(j in members | n[j])*constants.gas_constant*(T-constants.absolute_zero);
reconstruction reciprocal_family for reciprocal_reference(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy reference data_source=sum(j in members | n[j])*constants.gas_constant*(T-constants.absolute_zero);
fn forward_reduced(tau:Coordinate<forward_reference.tau>,delta:Coordinate<forward_reference.delta>,members:Set<chemistry.species>)->Reduced<forward_family>=tau^2*delta;
fn reciprocal_reduced(theta:Coordinate<reciprocal_reference.theta>,delta:Coordinate<reciprocal_reference.delta>,members:Set<chemistry.species>)->Reduced<reciprocal_family>=delta/theta^2;
fn forward(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy=reconstruct(forward_family,forward_reduced,T,V,n,members);
fn reciprocal(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy=reconstruct(reciprocal_family,reciprocal_reduced,T,V,n,members);
response chemical_response from law(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>,j:chemistry.species,law:Fn(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy)->LogFugacityCoefficient=partial(law,n[j])(T,V,n,members)/(constants.gas_constant*(T-constants.absolute_zero));
test composition_reference_partials {
let n[j in members]:Amount=if j==a then 0.3{mol} else 0.7{mol};
expect forward(300{K},0.1{m^3},n,members)==55451.069922606608{J} tolerance 1e-8{J};
expect reciprocal(300{K},0.1{m^3},n,members)==forward(300{K},0.1{m^3},n,members) tolerance 1e-8{J};
expect chemical_response(300{K},0.1{m^3},n,members,a,forward)==14.183431952662722{1} tolerance 1e-10{1};
expect chemical_response(300{K},0.1{m^3},n,members,a,reciprocal)==14.183431952662722{1} tolerance 1e-10{1};
expect helmholtz.isochoric_heat_capacity(300{K},0.1{m^3},n,members,forward)==-369.67379948404406{J/(mol*K)} tolerance 1e-8{J/(mol*K)};
expect helmholtz.isochoric_heat_capacity(300{K},0.1{m^3},n,members,reciprocal)==-369.67379948404406{J/(mol*K)} tolerance 1e-8{J/(mol*K)};
}
test ideal_physical_responses {
let n[j in members]:Amount=if j==a then 0.3{mol} else 0.7{mol};
expect helmholtz.ideal(300{K},0.025{m^3},n,members)==0{J} tolerance 1e-12{J};
expect helmholtz.pressure_response(300{K},0.025{m^3},n,members,helmholtz.ideal)==99773.55141783888{Pa} tolerance 1e-7{Pa};
expect helmholtz.enthalpy_response(300{K},0.025{m^3},n,members,helmholtz.ideal)==0{J/mol} tolerance 1e-10{J/mol};
expect helmholtz.entropy_response(300{K},0.025{m^3},n,members,helmholtz.ideal)==0{J/(mol*K)} tolerance 1e-10{J/(mol*K)};
expect helmholtz.isochoric_heat_capacity(300{K},0.025{m^3},n,members,helmholtz.ideal)==0{J/(mol*K)} tolerance 1e-10{J/(mol*K)};
expect helmholtz.isobaric_heat_capacity(300{K},0.025{m^3},n,members,helmholtz.ideal)==0{J/(mol*K)} tolerance 1e-10{J/(mol*K)};
expect helmholtz.fugacity_response(300{K},0.025{m^3},n,members,a,helmholtz.ideal)==0{1} tolerance 1e-12{1};
}
test nrtl_physical_homogeneity {
permission fit_use families(nrtl.parameters) allow_unknown true allow_extrapolation false;
let n[j in members]:Amount=if j==a then 0.3{mol} else 0.7{mol};
let triple[j in members]:Amount=3*n[j];
expect nrtl.normalized_excess(300{K},100000{Pa},n,members,parameters,nrtl.potential)==nrtl.binary(0.3,1,1,0.3,0.3) tolerance 1e-12;
// Independent binary closed-form derivatives at x=0.3, tau12=tau21=1, alpha12=alpha21=0.3.
expect nrtl.ln_gamma(300{K},100000{Pa},n,members,parameters,a)==0.8281233874524784{1} tolerance 1e-12{1};
expect nrtl.ln_gamma(300{K},100000{Pa},n,members,parameters,b)==0.15757658289359678{1} tolerance 1e-12{1};
expect nrtl.potential(300{K},100000{Pa},triple,members,parameters)==3*nrtl.potential(300{K},100000{Pa},n,members,parameters) tolerance 1e-8{J};
expect nrtl.ln_gamma(300{K},100000{Pa},triple,members,parameters,a)==nrtl.ln_gamma(300{K},100000{Pa},n,members,parameters,a) tolerance 1e-12{1};
expect nrtl.gibbs_duhem_response(300{K},100000{Pa},n,members,parameters,a,nrtl.potential)==0{1} tolerance 1e-12{1};
}
test pcsaft_physical_partials {
permission fit_use families(pcsaft_parameters.nonassociating,properties.predictive_rule) allow_unknown true allow_extrapolation false;
let n[j in members]:Amount=if j==a then 0.3{mol} else 0.7{mol};
let triple[j in members]:Amount=3*n[j];
expect saft(300{K},0.3{m^3},triple,members)==3*saft(300{K},0.1{m^3},n,members) tolerance 1e-8{J};
expect helmholtz.pressure_response(300{K},0.1{m^3},n,members,saft)==pressure_difference(300{K},0.1{m^3},n,members,1e-5{m^3},saft) tolerance 0.01{Pa};
expect helmholtz.isochoric_heat_capacity(300{K},0.1{m^3},n,members,saft)==cv_difference(300{K},0.1{m^3},n,members,0.01{K},saft) tolerance 0.001{J/(mol*K)};
expect helmholtz.fugacity_response(300{K},0.3{m^3},triple,members,a,saft)==helmholtz.fugacity_response(300{K},0.1{m^3},n,members,a,saft) tolerance 1e-10{1};
}
// Add use peng_robinson @"1.0.0"; use cubic @"1.0.0"; use pr_oracle @"1.0.0"; to FIXTURE.
// Independent 60-digit Decimal evaluation from the declared PR constants and
// asym_critical/directional_pair data: T=450 K, P=100000 Pa, rho=25 mol/m^3,
// x=(0.4,0.6), k_ab=0.1, k_ba=0.3. No production helper was evaluated.
test cubic_polynomial_frontdoors {
permission fit_use families(cubic.critical_point,cubic.directional_pair) allow_unknown true allow_extrapolation false;
let n[j in pr_oracle.asym_members]:Amount=if j==pr_oracle.asym_a then 0.4{mol} else 0.6{mol};
let triple[j in pr_oracle.asym_members]:Amount=3*n[j];
let x[j in pr_oracle.asym_members]:MoleFraction=if j==pr_oracle.asym_a then 0.4{1} else 0.6{1};
expect peng_robinson.density_residual(450{K},100000{Pa},25{mol/m^3},n,pr_oracle.asym_members,pr_oracle.asymmetric_parameters,cubic.pr)==6821.135953957148{Pa} tolerance 1e-7{Pa};
expect peng_robinson.density_residual(450{K},100000{Pa},25{mol/m^3},triple,pr_oracle.asym_members,pr_oracle.asymmetric_parameters,cubic.pr)==6821.135953957148{Pa} tolerance 1e-7{Pa};
expect peng_robinson.compressibility_residual(450{K},100000{Pa},0.97,x,pr_oracle.asym_members,pr_oracle.asymmetric_parameters,cubic.pr)==-0.024388804893713042 tolerance 1e-12;
expect peng_robinson.compressibility_residual(450{K},100000{Pa},0.004,x,pr_oracle.asym_members,pr_oracle.asymmetric_parameters,cubic.pr)==-0.0000115205266128288 tolerance 1e-12;
}

}
"#;
