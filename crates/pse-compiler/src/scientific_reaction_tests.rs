// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual reaction records, checked material projection and both authored reactor consumers.
#![allow(
    clippy::unwrap_used,
    reason = "focused checked scientific contract assertions"
)]
use crate::{
    authored_transfer_tests::{context, root, rows},
    workspace::{CompilerWorkspace, ModelingOutput, WorkspaceLimits},
};
use pse_kernels::DerivativeOrder;
use pse_math::{
    assembly::AssemblyLimits, binding::CaseValues, jets::EvaluationLimits, library::Optimization,
};
use pse_modeling::{
    Bindings, Limits, PhysicalScope,
    specialize::{SpecializedModel, root_instance},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};

const FIXTURE: &str = r#"package scientific_reaction_fixture {
use chemistry @"1.0.0"; use reaction_forms @"1.0.0"; use reactions @"1.0.0"; use references @"1.0.0";
use reactors @"1.0.0"; use saponification @"1.0.0"; use chem @"1.0.0";
use control_volumes @"1.0.0"; use equilibrium @"1.0.0"; use ciaaw @"1.0.0";
use correlations @"1.0.0"; use properties @"1.0.0"; use compatibility @"1.0.0";
set selected_species:Set<chemistry.species>=saponification.saponification_species;
test ConstantPoint {
 permission solvent_constants records(correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit],correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit]) allow_unknown true allow_extrapolation false;
 param cp:MolarCp=correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit].c0;
 param density:MolarDensity=correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit].c0;
 expect cp==75.327{J/(mol*K)} tolerance 1e-12{J/(mol*K)};
 expect density==55388{mol/m^3} tolerance 1e-9{mol/m^3};
}
test ProjectionPoint {
 permission kinetic_use families(reaction_forms.reaction_set) allow_unknown true allow_extrapolation false;
 child projection:reactions.Projection=reactions.Projection(material=selected_species,selected=saponification.SaponificationReactions.records,selection=saponification.SaponificationReactions.admitted);
}
test CstrPoint {
 // Standalone seed constants retain unknown form-signature evidence.
 permission solvent_constants records(correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit],correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit]) allow_unknown true allow_extrapolation false;
 permission kinetic_use families(reaction_forms.reaction_set) allow_unknown true allow_extrapolation false;
 child cstr:reactors.CSTR=reactors.CSTR(inlet_pkg=saponification.Saponification(defined_state=true,selected=selected_species),outlet_pkg=saponification.Saponification(defined_state=false,selected=selected_species),reaction_pkg=saponification.SaponificationReactions);
}
test PfrPoint {
 // Standalone seed constants retain unknown form-signature evidence.
 permission solvent_constants records(correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit],correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit]) allow_unknown true allow_extrapolation false;
 permission kinetic_use families(reaction_forms.reaction_set) allow_unknown true allow_extrapolation false;
 child pfr:reactors.PFR=reactors.PFR(inlet_pkg=saponification.Saponification(defined_state=true,selected=selected_species),outlet_pkg=saponification.Saponification(defined_state=false,selected=selected_species),reaction_pkg=saponification.SaponificationReactions,elements=1,use_radau=false);
}
test ElementalPoint:control_volumes.ControlVolume0D {
 // Standalone seed constants retain unknown form-signature evidence.
 permission solvent_constants records(correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit],correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit]) allow_unknown true allow_extrapolation false;
 override param material_basis:control_volumes.MaterialBalance=control_volumes.MaterialBalance.elementTotal;
 override param elements:Set<chemistry.element>={ciaaw.C};
 override param inlet_package:equilibrium.ThermoPackage=saponification.Saponification(defined_state=true,selected=selected_species);
 override param outlet_package:equilibrium.ThermoPackage=saponification.Saponification(defined_state=false,selected=selected_species);
}
test Both {
 // Standalone seed constants retain unknown form-signature evidence.
 permission solvent_constants records(correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit],correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit]) allow_unknown true allow_extrapolation false;
 permission kinetic_use families(reaction_forms.reaction_set) allow_unknown true allow_extrapolation false;
 child cstr:reactors.CSTR=reactors.CSTR(inlet_pkg=saponification.Saponification(defined_state=true,selected=selected_species),outlet_pkg=saponification.Saponification(defined_state=false,selected=selected_species),reaction_pkg=saponification.SaponificationReactions);
 child pfr:reactors.PFR=reactors.PFR(inlet_pkg=saponification.Saponification(defined_state=true,selected=selected_species),outlet_pkg=saponification.Saponification(defined_state=false,selected=selected_species),reaction_pkg=saponification.SaponificationReactions,elements=1,use_radau=false);
}
}"#;
// Complete production documents in the reaction consumers' imported dependency closure.
const REACTION_SOURCES: &[&str] = &[
    include_str!("../../../packages/reference/domain/models/numerical-policy.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/aqueous.pse"),
    include_str!("../../../packages/reference/methods/models/caloric.pse"),
    include_str!("../../../packages/reference/data/species/models/catalogue.pse"),
    include_str!("../../../packages/reference/physical/models/chemistry.pse"),
    include_str!("../../../packages/reference/data/ciaaw/models/catalogue.pse"),
    include_str!("../../../packages/reference/physical/models/compatibility.pse"),
    include_str!("../../../packages/reference/domain/models/constants.pse"),
    include_str!("../../../packages/reference/process/models/control-volumes.pse"),
    include_str!("../../../packages/reference/methods/models/correlations.pse"),
    include_str!("../../../packages/reference/process/models/control-volume-1d.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/equilibrium.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/helmholtz.pse"),
    include_str!("../../../packages/reference/physical/models/kinds.pse"),
    include_str!("../../../packages/reference/physical/models/math.pse"),
    include_str!("../../../packages/reference/domain/models/properties.pse"),
    include_str!("../../../packages/reference/domain/models/provenance.pse"),
    include_str!("../../../packages/reference/methods/models/pure-properties.pse"),
    include_str!("../../../packages/reference/methods/models/reaction-forms.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/reactions.pse"),
    include_str!("../../../packages/reference/process/models/reactors.pse"),
    include_str!("../../../packages/reference/data/references/models/references.pse"),
    include_str!("../../../packages/reference/seed-data/models/saponification.pse"),
];
fn source() -> String {
    format!("{}\n{FIXTURE}", REACTION_SOURCES.join("\n"))
}
fn specialize(text: &str, name: &str) -> Result<Arc<SpecializedModel>, String> {
    let declarations = rows(text);
    let id = root(&declarations, "scientific_reaction_fixture", name);
    let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .map_err(|e| e.to_string())?;
    workspace
        .specialize_modeling(
            id,
            root_instance(id),
            Bindings::default(),
            Limits::default(),
        )
        .map_err(|e| e.to_string())
}
fn contributions(text: &str, point: &str) -> Vec<(String, f64)> {
    let input = context();
    let registry = input.quantities.clone();
    let declarations = rows(text);
    let id = root(&declarations, "scientific_reaction_fixture", point);
    let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let prepared = workspace
        .prepare_modeling_cancellable(
            id,
            root_instance(id),
            Bindings::default(),
            Limits::default(),
            cancel.clone(),
        )
        .unwrap();
    let admitted = &prepared.admitted;
    let plan = admitted
        .plan(
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
    let assembly = Arc::new(
        plan.compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap(),
    );
    let values = CaseValues {
        scalars: admitted
            .inputs
            .iter()
            .map(|id| {
                let symbol = &prepared.model.symbols[id];
                let quantity = symbol
                    .ty
                    .quantity_scheme()
                    .unwrap()
                    .resolve(&registry, &Default::default())
                    .unwrap();
                let name = registry
                    .quantity_type(quantity)
                    .unwrap()
                    .name
                    .as_deref()
                    .unwrap();
                let value = match name {
                    "Temperature" => 303.15,
                    "Pressure" => 101325.0,
                    "MolarDensity" | "Density" => 100.0,
                    "Volume" | "Length" | "VolumeFlow" => 0.001,
                    "Area" => 1.0,
                    "MoleFraction" => 0.2,
                    "Flow" => 0.5,
                    _ => 0.0,
                };
                (*id, value)
            })
            .collect(),
    };
    let computed = assembly
        .worker(BTreeMap::new(), cancel)
        .constraints(&values)
        .unwrap();
    let ordered = admitted.ordered_values(&computed).unwrap();
    let mut result = Vec::new();
    for (output, value) in admitted.outputs.iter().zip(ordered) {
        let ModelingOutput::Contribution {
            accumulator,
            contribution,
        } = output
        else {
            continue;
        };
        let closure = &prepared.model.closures[accumulator];
        let term = closure
            .terms
            .iter()
            .find(|term| term.id == *contribution)
            .unwrap();
        let declaration = term.lineage.declaration.to_string();
        result.push((declaration, value));
    }
    result
}
fn reactor_sources(text: &str) -> BTreeMap<String, Vec<f64>> {
    let mut result: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for (declaration, value) in contributions(text, "Both") {
        if declaration == "7e7946177bf341dd8e5bcc6ded1b6913" {
            result.entry("cstr".into()).or_default().push(value);
        }
        if declaration == "4a54526c0701428b8e50134e16308634" {
            result.entry("pfr".into()).or_default().push(value);
        }
    }
    for values in result.values_mut() {
        values.sort_by(f64::total_cmp);
    }
    result
}

#[test]
fn scientific_reaction_authoritative_coefficients_change_both_reactor_source_vectors() {
    let text = source();
    let original = reactor_sources(&text);
    let doubled = text
        .replace(
            "[hydrolysis,chem.sodium_hydroxide]=[-1{1}]",
            "[hydrolysis,chem.sodium_hydroxide]=[-2{1}]",
        )
        .replace(
            "[hydrolysis,chem.ethyl_acetate]=[-1{1}]",
            "[hydrolysis,chem.ethyl_acetate]=[-2{1}]",
        )
        .replace(
            "[hydrolysis,chem.sodium_acetate]=[1{1}]",
            "[hydrolysis,chem.sodium_acetate]=[2{1}]",
        )
        .replace(
            "[hydrolysis,chem.ethanol]=[1{1}]",
            "[hydrolysis,chem.ethanol]=[2{1}]",
        );
    assert_ne!(text, doubled);
    let changed = reactor_sources(&doubled);
    let extent = 1.221230445517785;
    for (reactor, points) in [("cstr", 1), ("pfr", 2)] {
        let actual = &original[reactor];
        assert_eq!(actual.len(), 5 * points, "{reactor}");
        let mut expected = Vec::new();
        for _ in 0..points {
            expected.extend([-extent, -extent, 0.0, extent, extent]);
        }
        expected.sort_by(f64::total_cmp);
        for (actual, expected) in actual.iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1e-9,
                "{reactor}: {actual} != {expected}"
            );
        }
        for (original, changed) in actual.iter().zip(&changed[reactor]) {
            assert!((changed - 2.0 * original).abs() < 1e-9, "{reactor}");
        }
    }
}
#[test]
fn scientific_reaction_missing_products_refuse_both_reactors_before_solving() {
    let text = source().replace(
        "=saponification.saponification_species;",
        "={chem.sodium_hydroxide,chem.ethyl_acetate,chem.sodium_acetate,chem.water};",
    );
    for name in ["CstrPoint", "PfrPoint"] {
        let error = specialize(&text, name).unwrap_err();
        assert!(error.contains("MissingParticipant"), "{name}: {error}");
    }
}
#[test]
fn scientific_reaction_inert_extra_material_has_zero_authoritative_coefficient() {
    let text=source().replace("=saponification.saponification_species;","={chem.sodium_hydroxide,chem.ethyl_acetate,chem.sodium_acetate,chem.ethanol,chem.water,chem.nitrogen};");
    // Nitrogen is absent from the reaction and from the RateLaw component description.
    // Both concrete projections admit it through actual nonzero support, not set equality.
    for name in ["CstrPoint", "PfrPoint"] {
        specialize(&text, name).unwrap();
    }
    let numeric = reactor_sources(&text);
    assert_eq!(
        numeric["cstr"].iter().filter(|v| v.abs() < 1e-12).count(),
        2
    );
    assert_eq!(numeric["pfr"].iter().filter(|v| v.abs() < 1e-12).count(), 4);
}
#[test]
fn scientific_reaction_rate_and_heat_extent_identity_mismatch_refuse_independently() {
    let text=source().replace("entity chemistry.reaction hydrolysis {extent=hydrolysis_extent}","entity chemistry.reaction hydrolysis {extent=hydrolysis_extent}\nentity chemistry.extent_convention wrong_extent {normalization=chemistry.ExtentNormalization.AsAuthored}");
    for attribute in ["rate_extent", "heat_extent"] {
        let bad = text.replace(
            &format!("{attribute}=hydrolysis_extent"),
            &format!("{attribute}=wrong_extent"),
        );
        let error = specialize(&bad, "ProjectionPoint").unwrap_err();
        assert!(error.contains("ExtentMismatch"), "{attribute}: {error}");
    }
}
#[test]
fn scientific_reaction_missing_charge_cannot_establish_conservation() {
    let original = source();
    let text = original
        .lines()
        .map(|line| {
            if line.contains("entity chemistry.species sodium_hydroxide ") {
                line.replace("charge = 0{1}, ", "")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(text, source());
    let error = specialize(&text, "ProjectionPoint").unwrap_err();
    assert!(error.contains("IncompleteCharge"), "{error}");
}

#[test]
fn scientific_composition_elemental_control_volume_uses_guarded_authoritative_composition() {
    let text = source();
    let values = contributions(&text, "ElementalPoint")
        .into_iter()
        .filter(|(id, _)| id == "3179567b6666534189196d4d9a90eae9")
        .map(|(_, v)| v)
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 1);
    // 0.1 mol/s each of ester C4, acetate C2, ethanol C2; hydroxide/water C0.
    assert!((values[0] - 0.8).abs() < 1e-10, "{values:?}");
    // Water is not a reaction participant: the elemental model itself must refuse
    // its unknown vector even though the hydrolysis declaration remains admitted.
    let unknown = text
        .lines()
        .map(|line| {
            if line.contains("entity chemistry.species water ") {
                line.replace(
                    "composition = chemistry.CompositionKnowledge.Complete, ",
                    "",
                )
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(text, unknown);
    let error = specialize(&unknown, "ElementalPoint").unwrap_err();
    assert!(error.contains("IncompleteComposition"), "{error}");
}

#[test]
fn scientific_reaction_projection_cannot_be_replaced_by_an_interface_coefficient_default() {
    let text = source()
        + r#"
package invalid_projection {
 use reactions @"1.0.0";
 def Alternate:reactions.Projection {
  override let coefficient[r in reactions,j in components]:ReactionStoichiometry=0{1};
 }
}"#;
    let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
    let error = workspace
        .publish_modeling(rows(&text), PhysicalScope::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains("base must be an interface"), "{error}");
}
#[test]
fn scientific_reaction_multiple_selected_variants_refuse_one_rate_and_heat_vector() {
    let text = source()
        + r#"
package extra_kinetics {
 use reaction_forms @"1.0.0"; use references @"1.0.0";
 use saponification @"1.0.0"; use chem @"1.0.0";
 use provenance @"1.0.0";
 dataset duplicate:reaction_forms.arrhenius_second_order
 bind(parameterization=references.saponification_kinetics_fit,family=reaction_forms.kinetic_parameters,
      source=references.idaes_saponification_reactions,dependencies={},conventions={},
      rate_extent=saponification.hydrolysis_extent,heat_extent=saponification.hydrolysis_extent)
 provenance(references.idaes_saponification_reactions,provenance.Role.synthetic) {
 [saponification.hydrolysis,2]=[3132000{m^3/(mol*s)},43000{J/mol},-49000{J/mol},chem.sodium_hydroxide,chem.ethyl_acetate];
 }
}"#;
    let text=text.replace("selected=saponification.SaponificationReactions.records","selected={reaction_forms.arrhenius_second_order[references.saponification_kinetics_fit,reaction_forms.kinetic_parameters,saponification.hydrolysis],reaction_forms.arrhenius_second_order[references.saponification_kinetics_fit,reaction_forms.kinetic_parameters,saponification.hydrolysis,2]}");
    let error = specialize(&text, "ProjectionPoint").unwrap_err();
    assert!(
        error.contains("exactly one kinetic and heat record per reaction")
            || error.contains("UnadmittedSelection"),
        "{error}"
    );
}

const COSTING_FIXTURE: &str = r#"package scientific_costing_fixture {
 use costing @"1.0.0";
 test reported_inside {
  expect costing.pressure_factor(5)==1.1128 tolerance 1e-12;
  expect costing.oversize_factor(1.1)==1.1 tolerance 1e-12;
 }
 test pressure_refused {expect costing.pressure_factor(0)==0.9803 tolerance 1e-12;}
 test pressure_permitted {
  permission pressure_use families(costing.pressure_factor) allow_unknown false allow_extrapolation true;
  expect costing.pressure_factor(0)==0.9803 tolerance 1e-12;
 }
 test wrong_permission {
  permission other_use families(costing.oversize_factor) allow_unknown false allow_extrapolation true;
  expect costing.pressure_factor(0)==0.9803 tolerance 1e-12;
 }
 test area_refused {expect costing.base_amount(11840.3014583805,11.3852,0.9186,0.0979)==87704.5883652126 tolerance 1;}
 test area_permitted {
  permission area_use families(costing.base_amount,costing.material_amount) allow_unknown true allow_extrapolation false;
  expect costing.base_amount(11840.3014583805,11.3852,0.9186,0.0979)==87704.5883652126 tolerance 1;
  expect costing.material_amount(10763.910416709722,costing.stainless_stainless)==4.08752 tolerance 0.00005;
 }
 test hard_domain {
  permission area_use families(costing.base_amount) allow_unknown true allow_extrapolation true;
  expect costing.base_amount(0,11.3852,0.9186,0.0979)==0 tolerance 1;
 }
}"#;
fn costing_point(name: &str) -> Result<(), String> {
    let text = format!(
        "{}\n{}\n{}\n{COSTING_FIXTURE}",
        [
            include_str!("../../../packages/reference/physical/models/chemistry.pse"),
            include_str!("../../../packages/reference/physical/models/compatibility.pse"),
            include_str!("../../../packages/reference/domain/models/provenance.pse"),
            include_str!("../../../packages/reference/domain/models/constants.pse"),
            include_str!("../../../packages/reference/domain/models/properties.pse"),
            include_str!("../../../packages/reference/data/references/models/references.pse"),
        ]
        .join("\n"),
        include_str!("../../../packages/reference/process/models/costing.pse"),
        include_str!("../../../packages/reference/seed-data/models/sslw.pse")
    );
    let declarations = rows(&text);
    let id = root(&declarations, "scientific_costing_fixture", name);
    let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .map_err(|e| e.to_string())?;
    let result = workspace
        .check_modeling_point(
            id,
            root_instance(id),
            Bindings::default(),
            Limits::default(),
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            crate::workspace::Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .map_err(|e| e.to_string())?;
    assert!(!result.expectations.is_empty());
    assert!(
        result.expectations.iter().all(|v| v.passed),
        "{name}: {result:?}"
    );
    Ok(())
}
#[test]
fn scientific_costing_reported_intervals_refuse_without_exact_named_permission() {
    costing_point("reported_inside").unwrap();
    assert!(costing_point("pressure_refused").is_err());
    costing_point("pressure_permitted").unwrap();
    assert!(costing_point("wrong_permission").is_err());
}
#[test]
fn scientific_costing_unknown_area_requires_permission_and_never_permits_zero_area() {
    assert!(costing_point("area_refused").is_err());
    costing_point("area_permitted").unwrap();
    let error = costing_point("hard_domain").unwrap_err();
    assert!(
        error.contains("valid") || error.contains("guard") || error.contains("domain"),
        "{error}"
    );
}

#[test]
fn scientific_reaction_records_require_admitted_closure_and_participant_scope() {
    let original = source();
    let unadmitted=original.replace("roots={reaction_forms.arrhenius_second_order[references.saponification_kinetics_fit,reaction_forms.kinetic_parameters,hydrolysis]}","roots={}");
    assert_ne!(original, unadmitted);
    let error = specialize(&unadmitted, "ProjectionPoint").unwrap_err();
    assert!(error.contains("UnadmittedSelection"), "{error}");
    let incomplete=original.replace("subjects={chem.sodium_hydroxide,chem.ethyl_acetate,chem.sodium_acetate,chem.ethanol},models={reaction_forms.kinetic_parameters}","subjects={chem.sodium_hydroxide,chem.ethyl_acetate,chem.sodium_acetate},models={reaction_forms.kinetic_parameters}");
    assert_ne!(original, incomplete);
    let error = specialize(&incomplete, "ProjectionPoint").unwrap_err();
    assert!(error.contains("MissingScientificParticipant"), "{error}");
}

#[test]
fn scientific_seed_constant_parameter_reads_require_exact_unknown_permission() {
    fn point(text: &str) -> Result<crate::workspace::ModelingPointChecks, String> {
        let declarations = rows(text);
        let id = root(
            &declarations,
            "scientific_reaction_fixture",
            "ConstantPoint",
        );
        let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
        workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .map_err(|e| e.to_string())?;
        workspace
            .check_modeling_point(
                id,
                root_instance(id),
                Bindings::default(),
                Limits::default(),
                &CaseValues {
                    scalars: BTreeMap::new(),
                },
                crate::workspace::Profile::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .map_err(|e| e.to_string())
    }
    let text = source();
    let permission = "permission solvent_constants records(correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit],correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit]) allow_unknown true allow_extrapolation false;";
    assert!(text.contains(permission));
    for replacement in [
        "",
        "permission solvent_constants records(correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit],correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit]) allow_unknown false allow_extrapolation true;",
    ] {
        let error = point(&text.replace(permission, replacement)).unwrap_err();
        assert!(error.contains("applicability"), "{error}");
    }
    let checks = point(&text).unwrap();
    assert!(checks.expectations.iter().all(|e| e.passed));
    use pse_model::generated::enums::ModelingApplicabilityOutcome as O;
    assert_eq!(checks.applicability.len(), 2);
    assert!(checks.applicability.iter().all(|o| o.required
        && o.outcome == O::UnknownEvidence
        && o.unknown_allowed
        && !o.extrapolation_allowed));
}
