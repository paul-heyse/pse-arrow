// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated authored-potential admission and library evaluation, without storage or solvers.
#![allow(
    clippy::unwrap_used,
    reason = "focused physical-potential unit assertions"
)]

use crate::workspace::{CompilerWorkspace, Inputs, Profile, WorkspaceLimits};
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
    let input = Inputs {
        quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
        preconditions: Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        ),
        flows: BTreeMap::new(),
        definitions: BTreeMap::new(),
        domains: BTreeMap::new(),
        groups: BTreeMap::new(),
        providers: BTreeMap::new(),
        cases: BTreeMap::new(),
        values: BTreeMap::new(),
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
            Limits::default(),
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

const FIXTURE: &str = r#"
package physical_potential_fixture {
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
entity properties.property_package parameters {}
dataset nrtl_parameters:nrtl.parameters provenance(data_source,provenance.Role.synthetic) {
[data_source,a,a]=[0,0]; [data_source,a,b]=[1,0.3];
[data_source,b,a]=[1,0.3]; [data_source,b,b]=[0,0];
}
dataset selected_pairs:interactions.pair_selection provenance(data_source,provenance.Role.synthetic) {
[parameters,nrtl.interaction_parameters]=[data_source];
[parameters,pcsaft_parameters.binary_interaction]=[data_source];
}
dataset pair_policy:interactions.absence_policy provenance(data_source,provenance.Role.synthetic) {
[parameters,pcsaft_parameters.binary_interaction]=[interactions.PairAbsence.predictiveZero];
}
// Published Gross-Sadowski methane and ethane parameters; no Arrow/storage journey is needed.
dataset segments:pcsaft_parameters.nonassociating bind(property=pcsaft_parameters.segments,phase_type=compatibility.PhaseType.vaporPhase,source=data_source) provenance(data_source,provenance.Role.synthetic) {
[a]=[1,3.7039e-10{m},150.03{K}]; [b]=[1.6069,3.5206e-10{m},191.42{K}];
}
dataset selection:properties.selection provenance(data_source,provenance.Role.synthetic) {
[parameters,a,pcsaft_parameters.segments,compatibility.PhaseType.vaporPhase]=[pcsaft_parameters.nonassociating[a,pcsaft_parameters.segments,compatibility.PhaseType.vaporPhase,data_source]];
[parameters,b,pcsaft_parameters.segments,compatibility.PhaseType.vaporPhase]=[pcsaft_parameters.nonassociating[b,pcsaft_parameters.segments,compatibility.PhaseType.vaporPhase,data_source]];
}
fn saft(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy=pcsaft.potential(T,V,n,members,parameters);
response pressure_difference from law(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>,dv:Volume,law:Fn(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy)->Pressure=sum(j in members | n[j])*constants.gas_constant*(T-constants.absolute_zero)/V-(law(T,V+dv,n,members)-law(T,V-dv,n,members))/(2*dv);
response cv_difference from law(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>,dt:TemperatureScale,law:Fn(T:Temperature,V:Volume,n:Amount[chemistry.species],members:Set<chemistry.species>)->ResidualHelmholtzEnergy)->MolarCp=-(T-constants.absolute_zero)*(law(T+dt,V,n,members)-2*law(T,V,n,members)+law(T-dt,V,n,members))/(dt^2*sum(j in members | n[j]));
test ideal_physical_responses {
let n[j in members]:Amount=if j==a then 0.3{mol} else 0.7{mol};
expect helmholtz.ideal(300{K},0.025{m^3},n,members)==0{J} tolerance 1e-12{J};
expect helmholtz.pressure_response(300{K},0.025{m^3},n,members,helmholtz.ideal)==99773.55141783888{Pa} tolerance 1e-7{Pa};
expect helmholtz.enthalpy_response(300{K},0.025{m^3},n,members,helmholtz.ideal)==0{J/mol} tolerance 1e-10{J/mol};
expect helmholtz.entropy_response(300{K},0.025{m^3},n,members,helmholtz.ideal)==0{J/(mol*K)} tolerance 1e-10{J/(mol*K)};
expect helmholtz.isochoric_heat_capacity(300{K},0.025{m^3},n,members,helmholtz.ideal)==0{J/(mol*K)} tolerance 1e-10{J/(mol*K)};
expect helmholtz.isobaric_heat_capacity(300{K},0.025{m^3},n,members,helmholtz.ideal)==0{J/(mol*K)} tolerance 1e-10{J/(mol*K)};
expect helmholtz.fugacity_response(300{K},0.025{m^3},n,members,a,helmholtz.ideal)==0 tolerance 1e-12;
}
test nrtl_physical_homogeneity {
let n[j in members]:Amount=if j==a then 0.3{mol} else 0.7{mol};
let triple[j in members]:Amount=3*n[j];
expect nrtl.normalized_excess(300{K},100000{Pa},n,members,parameters,nrtl.potential)==nrtl.binary(0.3,1,1,0.3,0.3) tolerance 1e-12;
expect nrtl.potential(300{K},100000{Pa},triple,members,parameters)==3*nrtl.potential(300{K},100000{Pa},n,members,parameters) tolerance 1e-8{J};
expect nrtl.ln_gamma(300{K},100000{Pa},triple,members,parameters,a)==nrtl.ln_gamma(300{K},100000{Pa},n,members,parameters,a) tolerance 1e-12;
expect nrtl.gibbs_duhem_response(300{K},100000{Pa},n,members,parameters,a,nrtl.potential)==0 tolerance 1e-12;
}
test pcsaft_physical_partials {
let n[j in members]:Amount=if j==a then 0.3{mol} else 0.7{mol};
let triple[j in members]:Amount=3*n[j];
expect saft(300{K},0.3{m^3},triple,members)==3*saft(300{K},0.1{m^3},n,members) tolerance 1e-8{J};
expect helmholtz.pressure_response(300{K},0.1{m^3},n,members,saft)==pressure_difference(300{K},0.1{m^3},n,members,1e-5{m^3},saft) tolerance 0.01{Pa};
expect helmholtz.isochoric_heat_capacity(300{K},0.1{m^3},n,members,saft)==cv_difference(300{K},0.1{m^3},n,members,0.01{K},saft) tolerance 0.001{J/(mol*K)};
expect helmholtz.fugacity_response(300{K},0.3{m^3},triple,members,a,saft)==helmholtz.fugacity_response(300{K},0.1{m^3},n,members,a,saft) tolerance 1e-10;
}
}
"#;
