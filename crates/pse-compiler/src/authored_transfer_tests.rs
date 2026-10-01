// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual authored process/reference consumers, isolated from storage and solver journeys.
#![allow(
    clippy::unwrap_used,
    reason = "focused authored physical contract assertions"
)]

use crate::workspace::{CompilerWorkspace, Inputs, ModelingOutput, Profile, WorkspaceLimits};
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use pse_math::{
    assembly::AssemblyLimits, binding::CaseValues, jets::EvaluationLimits, library::Optimization,
};
use pse_modeling::{
    Bindings, Declaration, DeclarationId, Limits, PhysicalScope, specialize::root_instance,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};

/// Read current source documents, including the process consumers and both caloric bindings.
/// External data transport occurrences are outside this selected declaration control; numerical
/// cases supply small synthetic keyed parameter rows for the selected declarations. Full package/data transport is
/// qualified in Plan 25k. No source function or process equation is copied into this test.
pub(crate) fn reference_sources() -> String {
    format!("{}\n{SYNTHETIC_PARAMETERS}", SOURCES.join("\n"))
}

// Complete source documents for the changed packages and their imported dependency closure.
// Keep this explicit: adding an unrelated reference package does not broaden these controls.
const SOURCES: &[&str] = &[
    include_str!("../../../packages/reference/thermodynamics/models/aqueous.pse"),
    include_str!("../../../packages/reference/seed-data/models/bt-ideal.pse"),
    include_str!("../../../packages/reference/seed-data/models/bt-pr.pse"),
    include_str!("../../../packages/reference/methods/models/caloric.pse"),
    include_str!("../../../packages/reference/seed-data/models/caloric.pse"),
    include_str!("../../../packages/reference/data/species/models/catalogue.pse"),
    include_str!("../../../packages/reference/physical/models/chemistry.pse"),
    include_str!("../../../packages/reference/data/ciaaw/models/catalogue.pse"),
    include_str!("../../../packages/reference/physical/models/compatibility.pse"),
    include_str!("../../../packages/reference/domain/models/constants.pse"),
    include_str!("../../../packages/reference/process/models/control-volumes.pse"),
    include_str!("../../../packages/reference/process/models/separator.pse"),
    include_str!("../../../packages/reference/methods/models/correlations.pse"),
    include_str!("../../../packages/reference/methods/models/cubic.pse"),
    include_str!("../../../packages/reference/process/models/control-volume-1d.pse"),
    include_str!("../../../packages/reference/seed-data/models/eos-data.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/equilibrium.pse"),
    include_str!("../../../packages/reference/process/models/heat-exchanger.pse"),
    include_str!("../../../packages/reference/process/models/homogeneous-units.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/phase-stability.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/helmholtz.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/ideal-caloric-state.pse"),
    include_str!("../../../packages/reference/process/models/pressure-changer.pse"),
    include_str!("../../../packages/reference/seed-data/models/pressure-fixtures.pse"),
    include_str!("../../../packages/reference/data/oracles/idaes-2.13/models/parameters.pse"),
    include_str!("../../../packages/reference/domain/models/interactions.pse"),
    include_str!("../../../packages/reference/physical/models/kinds.pse"),
    include_str!("../../../packages/reference/physical/models/math.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/nested-equilibrium.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/peng-robinson.pse"),
    include_str!("../../../packages/reference/seed-data/models/phase-data.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/pr-equilibrium.pse"),
    include_str!("../../../packages/reference/domain/models/properties.pse"),
    include_str!("../../../packages/reference/domain/models/provenance.pse"),
    include_str!("../../../packages/reference/methods/models/pure-properties.pse"),
    include_str!("../../../packages/reference/methods/models/reaction-forms.pse"),
    include_str!("../../../packages/reference/thermodynamics/models/reactions.pse"),
    include_str!("../../../packages/reference/process/models/reactors.pse"),
    include_str!("../../../packages/reference/data/references/models/references.pse"),
    include_str!("../../../packages/reference/seed-data/models/saponification.pse"),
    include_str!("../../../packages/reference/process/models/delta-t.pse"),
    include_str!("../../../packages/reference/process/models/units.pse"),
    include_str!("../../../packages/reference/process/models/vessels.pse"),
];

pub(crate) fn inputs() -> Inputs {
    Inputs {
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
    }
}

pub(crate) fn rows(text: &str) -> Vec<Declaration> {
    let parsed = parse(
        text,
        SemanticId::from_bytes([194; 16]),
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .unwrap();
    // Data transport is not this compiler control's scope. Keep scientific declarations
    // byte-derived from the current documents, while omitting external dataset bindings
    // and their derived dataset occurrences. Numerical cases supply selected synthetic data.
    let packages = parsed
        .iter()
        .filter(|row| row.parent_id.is_none())
        .map(|row| (row.declaration_id, row.name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut excluded = parsed
        .iter()
        .filter(|row| {
            row.value
                .dataset
                .as_ref()
                .is_some_and(|dataset| dataset.document.is_some())
        })
        .map(|row| row.declaration_id)
        .collect::<std::collections::BTreeSet<_>>();
    loop {
        let names = parsed
            .iter()
            .filter(|row| excluded.contains(&row.declaration_id))
            .filter_map(|row| Some(format!("{}.{}", packages.get(&row.parent_id?)?, row.name)))
            .collect::<std::collections::BTreeSet<_>>();
        let before = excluded.len();
        for row in &parsed {
            if row.value.dataset.as_ref().is_some_and(|dataset| {
                dataset
                    .provenance
                    .lineage
                    .iter()
                    .any(|lineage| names.contains(&lineage.path.join(".")))
            }) {
                excluded.insert(row.declaration_id);
            }
        }
        if excluded.len() == before {
            break;
        }
    }
    parsed
        .into_iter()
        .filter(|row| !excluded.contains(&row.declaration_id))
        .collect()
}

pub(crate) fn root(rows: &[Declaration], package: &str, name: &str) -> DeclarationId {
    let parent = rows
        .iter()
        .find(|row| row.name == package && row.parent_id.is_none())
        .unwrap()
        .declaration_id;
    rows.iter()
        .find(|row| row.name == name && row.parent_id == Some(parent))
        .unwrap()
        .declaration_id
}

fn workspace(text: &str) -> (CompilerWorkspace, Vec<Declaration>) {
    let rows = rows(text);
    let mut workspace = CompilerWorkspace::new(inputs(), WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(rows.clone(), PhysicalScope::default())
        .unwrap();
    (workspace, rows)
}

fn check(name: &str) {
    let (mut workspace, rows) = workspace(&format!("{}\n{FIXTURE}", reference_sources()));
    let root = root(&rows, "authored_transfer_fixture", name);
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
        result.expectations.iter().all(|value| value.passed),
        "{name}: {result:?}"
    );
}

#[test]
fn authored_process_and_caloric_declarations_admit() {
    let _ = workspace(&reference_sources());
}

#[test]
fn authored_bt_pr_translations_preserve_composition_pressure_and_inverse() {
    check("pr_translations");
}

#[test]
fn authored_bt_ideal_translation_uses_selected_caloric_anchor_and_inverse() {
    check("ideal_translation");
}

#[test]
fn authored_ideal_entropy_correction_retains_pressure_composition_and_caloric_roles() {
    check("ideal_entropy_correction");
}

#[test]
fn authored_material_projections_keep_component_and_element_contracts() {
    check("material_projections");
}

#[test]
fn authored_cstr_stored_energy_retains_stock_pressure_and_enthalpy_origins() {
    check("stored_energy");
}

#[test]
fn authored_reaction_extent_and_stoichiometric_component_rates_are_physical() {
    check("reaction_rates");
}

#[test]
fn authored_vessel_amount_and_energy_equations_evaluate_without_adapters() {
    let input = inputs();
    let registry = input.quantities.clone();
    let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    let rows = rows(&format!("{}\n{FIXTURE}", reference_sources()));
    let root = root(&rows, "authored_transfer_fixture", "InventoryPoint");
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            root_instance(root),
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
    // Independent ideal-gas calculation: N=rho*V=1 mol and U=N*h-p*V.
    // The authored equations themselves, including both vessel closures, are evaluated below.
    let energy = 2000.0 - 10.0 * 8.314_462_618_153_24 * 300.0 * 0.1;
    let values = CaseValues {
        scalars: admitted
            .inputs
            .iter()
            .map(|id| {
                let path = &prepared.model.symbols[id].lineage.path;
                let value = if path.ends_with(".phase.T") {
                    300.0
                } else if path.ends_with(".phase.rho") {
                    10.0
                } else if path.contains(".phase.amount") {
                    0.5
                } else if path.ends_with(".amount") {
                    1.0
                } else if path.ends_with(".root.volume") {
                    0.1
                } else if path.ends_with(".energy") {
                    energy
                } else {
                    panic!("unexpected vessel input: {path}")
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
    let mut equations = 0;
    for (output, value) in admitted.outputs.iter().zip(ordered) {
        if let ModelingOutput::Equation { id, .. } = output {
            equations += 1;
            let path = &prepared
                .model
                .equations
                .iter()
                .find(|row| row.id == *id)
                .unwrap()
                .lineage
                .path;
            assert!(value.abs() < 1e-8, "{path}: residual {value}");
        }
    }
    assert_eq!(
        equations, 4,
        "two component bindings plus amount and energy closures"
    );
}

#[test]
fn authored_cstr_extent_report_retains_quantity_and_unit_contract() {
    let (mut workspace, rows) = workspace(&format!("{}\n{FIXTURE}", reference_sources()));
    let root = root(&rows, "authored_transfer_fixture", "CstrPoint");
    let model = workspace
        .specialize_modeling(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let mut reported = Vec::new();
    for annotation in &model.annotations {
        if let pse_modeling::annotation::AnnotationValue::Report(label) = &annotation.value
            && label == "reaction extent in mol/s"
        {
            reported.push(&model.symbols[&annotation.target]);
        }
    }
    assert_eq!(reported.len(), 1);
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let expected = registry
        .quantity_types()
        .find(|quantity| quantity.name.as_deref() == Some("ReactionExtentRate"))
        .unwrap();
    let scheme = reported[0].ty.quantity_scheme().unwrap();
    assert_eq!(
        scheme
            .resolve(&registry, &pse_quantity::scheme::Substitution::default())
            .unwrap(),
        expected.id
    );
    assert_eq!(
        registry.unit(expected.canonical_unit).unwrap().symbol,
        "mol/s"
    );
    assert!(reported[0].ty.physical_refinement().is_none());
}

#[test]
fn authored_distributed_energy_boundaries_bind_each_actual_coordinate() {
    let (mut workspace, rows) = workspace(&format!("{}\n{FIXTURE}", reference_sources()));
    let root = root(&rows, "authored_transfer_fixture", "DistributedPoint");
    let model = workspace
        .specialize_modeling(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let closures = model
        .closures
        .values()
        .filter(|closure| closure.lineage.path.contains("energy_source"))
        .collect::<Vec<_>>();
    assert_eq!(
        closures.len(),
        2,
        "one finite spatial element has two energy boundaries"
    );
    let mut boundaries = Vec::new();
    for closure in closures {
        let boundary = closure.boundary.as_ref().unwrap();
        assert!(boundary.is_bound());
        let pse_modeling::BoundaryRef::Bound { coordinates, .. } = boundary else {
            panic!("energy boundary is not bound")
        };
        assert_eq!(coordinates.len(), 1);
        assert_eq!(closure.terms.len(), 3, "heat, work and reaction energy");
        boundaries.push(boundary);
    }
    assert_ne!(boundaries[0], boundaries[1]);
    let reports = model
        .annotations
        .iter()
        .filter_map(|annotation| match &annotation.value {
            pse_modeling::annotation::AnnotationValue::Report(label)
                if label == "distributed heat" =>
            {
                Some(&model.symbols[&annotation.target])
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(reports.len(), 2);
    let registry = pse_quantity::standard::standard_registry().unwrap();
    for symbol in reports {
        let Some(pse_modeling::PhysicalRefinement::Transfer {
            boundary,
            direction,
        }) = symbol.ty.physical_refinement()
        else {
            panic!("distributed report lost transfer context")
        };
        assert_eq!(*direction, pse_modeling::TransferDirection::Into);
        assert!(boundaries.contains(&boundary));
        let quantity = symbol
            .ty
            .quantity_scheme()
            .unwrap()
            .resolve(&registry, &pse_quantity::scheme::Substitution::default())
            .unwrap();
        let contract = registry.quantity_type(quantity).unwrap();
        assert_eq!(contract.name.as_deref(), Some("EnergyTransferRate"));
        assert!(contract.key.reference_state.is_none());
        assert_eq!(registry.unit(contract.canonical_unit).unwrap().symbol, "W");
    }
}

#[test]
fn authored_exchanger_reflection_preserves_opposite_actual_owners() {
    let (mut workspace, rows) = workspace(&format!("{}\n{FIXTURE}", reference_sources()));
    let root = root(&rows, "authored_transfer_fixture", "ExchangerPoint");
    let model = workspace
        .specialize_modeling(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    assert_eq!(model.exchanges.len(), 1);
    let pair = model.exchanges.values().next().unwrap();
    assert!(pair.first.is_bound() && pair.second.is_bound());
    assert_ne!(pair.first, pair.second);
    assert!(model.functions.values().any(|function| matches!(
        function.physical_operation,
        Some(pse_modeling::PhysicalOperation::Transfer {
            factor: -1,
            exchange: Some(_),
            ..
        })
    )));
    let energy = model
        .closures
        .values()
        .filter(|closure| closure.lineage.path.ends_with(".energy"))
        .collect::<Vec<_>>();
    assert_eq!(energy.len(), 2);
    assert!(energy.iter().all(|closure| {
        closure
            .boundary
            .as_ref()
            .is_some_and(|boundary| boundary == &pair.first || boundary == &pair.second)
    }));
}

/// Exercise source-language misuse through check AND actual-owner specialization, rather than
/// asserting only the helper admission predicate.
fn refusal(text: &str) -> String {
    let rows = rows(text);
    let root = root(&rows, "p", "Root");
    let mut workspace = CompilerWorkspace::new(inputs(), WorkspaceLimits::default()).unwrap();
    if let Err(error) = workspace.publish_modeling(rows, PhysicalScope::default()) {
        return error.to_string();
    }
    workspace
        .specialize_modeling(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap_err()
        .to_string()
}

#[test]
fn authored_transfers_refuse_wrong_owner_coordinate_and_second_sign_application() {
    for (label, text) in [
        (
            "owner",
            r#"package p { def Cell {boundary wall; var heat:Transfer<EnergyTransferRate,wall,Into>;} def Root {child a:Cell=Cell(); child b:Cell=Cell(); eq bad:a.heat==b.heat;} }"#,
        ),
        (
            "coordinate",
            r#"package p {def Root {entity kind position {} entity position a {} entity position b {} set positions:Set<position>={a,b}; boundary wall[i in positions]; var heat[i in positions]:Transfer<EnergyTransferRate,wall,Into>; eq bad:heat[a]==heat[b];}}"#,
        ),
        (
            "second sign",
            r#"package p {def Root {boundary wall; let q:EnergyTransferRate=25{W}; let heat:Transfer<EnergyTransferRate,wall,Into>=transfer(q,wall,Into); accumulate ledger:Power boundary wall accounting tolerance 1e-9{W}; contribute ledger role negative=heat;}}"#,
        ),
        (
            "unconverted removal",
            r#"package p {def Root {boundary wall; let q:EnergyTransferRate=25{W}; let removal:Transfer<EnergyTransferRate,wall,OutOf>=transfer(q,wall,OutOf); accumulate ledger:Power boundary wall accounting tolerance 1e-9{W}; contribute ledger role directed=removal;}}"#,
        ),
        (
            "double wrapping",
            r#"package p {def Root {boundary wall; let q:EnergyTransferRate=25{W}; let heat:Transfer<EnergyTransferRate,wall,Into>=transfer(q,wall,Into); let twice:Transfer<EnergyTransferRate,wall,Into>=transfer(heat,wall,Into);}}"#,
        ),
    ] {
        let reason = refusal(text);
        assert!(
            reason.contains("transfer")
                || reason.contains("boundary")
                || reason.contains("directed")
                || reason.contains("contextual owners"),
            "{label}: unrelated refusal: {reason}"
        );
    }
}

const SYNTHETIC_PARAMETERS: &str = r#"
package transfer_parameters {
use chemistry @"1.0.0"; use chem @"1.0.0"; use provenance @"1.0.0";
use references @"1.0.0"; use correlations @"1.0.0"; use properties @"1.0.0";
use compatibility @"1.0.0"; use cubic @"1.0.0"; use idaes_thermo @"1.0.0";
// Simple independent caloric data exercise the actual BTIdeal property-selection and anchor
// functions. These synthetic coefficients are not an IDAES or database qualification.
dataset cp:correlations.rpp4_cp bind(property=properties.heat_capacity,phase_type=compatibility.PhaseType.vaporPhase,source=references.idaes_bt_ideal,parameterization=references.idaes_bt_ideal_vapor_fit,dependencies={},conventions={}) provenance(references.analytic_identities,provenance.Role.synthetic) {
[chem.benzene]=[200{K},500{K},10{J/(mol*K)},0{J/(mol*K^2)},0{J/(mol*K^3)},0{J/(mol*K^4)}];
[chem.toluene]=[200{K},500{K},20{J/(mol*K)},0{J/(mol*K^2)},0{J/(mol*K^3)},0{J/(mol*K^4)}];
}
// These unused liquid, pressure and critical parameter rows satisfy the actual seed's
// selection declarations. They retain those keys without importing the data transport journey.
dataset liquid_cp:correlations.dippr100 bind(property=properties.heat_capacity,phase_type=compatibility.PhaseType.liquidPhase,source=references.idaes_bt_ideal,parameterization=references.idaes_bt_ideal_liquid_fit,dependencies={},conventions={}) provenance(references.analytic_identities,provenance.Role.synthetic) {
[chem.benzene]=[200{K},500{K},10{J/(mol*K)},0{J/(mol*K^2)},0{J/(mol*K^3)},0{J/(mol*K^4)},0{J/(mol*K^5)}];
[chem.toluene]=[200{K},500{K},20{J/(mol*K)},0{J/(mol*K^2)},0{J/(mol*K^3)},0{J/(mol*K^4)},0{J/(mol*K^5)}];
}
dataset pressure:correlations.rpp4_wagner bind(property=properties.vapor_pressure,phase_type=compatibility.PhaseType.liquidPhase,source=references.idaes_test_rpp4,parameterization=references.idaes_rpp4_pressure_fit,dependencies={},conventions={}) provenance(references.analytic_identities,provenance.Role.synthetic) {
[chem.benzene]=[200{K},500{K},0{1},0{1},0{1},0{1},600{K},5000000{Pa}];
[chem.toluene]=[200{K},500{K},0{1},0{1},0{1},0{1},600{K},5000000{Pa}];
}
dataset critical:cubic.critical_point bind(property=cubic.critical_parameters,source=references.idaes_bt_pr,parameterization=references.idaes_bt_pr_critical_fit,dependencies={},conventions={}) provenance(references.analytic_identities,provenance.Role.synthetic) {
[chem.benzene]=[600{K},5000000{Pa},0.1{1}];
[chem.toluene]=[600{K},5000000{Pa},0.1{1}];
}
// Unused observation placeholders satisfy declared complete tables after external transport
// exclusion. This control never evaluates an oracle fixture against these synthetic values.
dataset observation_state:idaes_thermo.srk_state provenance(references.analytic_identities,provenance.Role.synthetic) {
[0]=[300{K},10{mol/m^3},100000{Pa},1{1}];
}
dataset observation_components:idaes_thermo.srk_component provenance(references.analytic_identities,provenance.Role.synthetic) {
[chem.benzene]=[0.5{1},0{1}];
[chem.toluene]=[0.5{1},0{1}];
}
}
"#;

const FIXTURE: &str = r#"
package authored_transfer_fixture {
use equilibrium @"1.0.0";
use pressure_fixtures @"1.0.0";
use control_volumes @"1.0.0";
use ciaaw @"1.0.0";
use bt_pr @"1.0.0"; use bt_ideal @"1.0.0"; use constants @"1.0.0";
use chemistry @"1.0.0"; use chem @"1.0.0"; use provenance @"1.0.0";
use references @"1.0.0"; use correlations @"1.0.0"; use properties @"1.0.0";
use compatibility @"1.0.0"; use reactors @"1.0.0"; use vessels @"1.0.0";
use helmholtz @"1.0.0"; use saponification @"1.0.0"; use reactions @"1.0.0"; use heat_exchange @"1.0.0";
use reaction_forms @"1.0.0";
reference translation pr_h_inverse(value:PrOracleEnthalpy,composition:MoleFraction[chemistry.species],members:Set<chemistry.species>)->MolarEnthalpy anchors(source=bt_pr.oracle_enthalpy_anchor,target=bt_pr.stock_enthalpy_anchor) at(temperature=bt_pr_oracle.temperature,pressure=bt_pr_oracle.pressure) provenance(references.idaes_bt_pr,provenance.Role.oracle_input);
reference translation pr_s_inverse(value:PrOracleEntropy,composition:MoleFraction[chemistry.species],members:Set<chemistry.species>)->MolarEntropy anchors(source=bt_pr.oracle_entropy_anchor,target=bt_pr.stock_entropy_anchor) at(temperature=bt_pr_oracle.temperature,pressure=bt_pr_oracle.pressure) provenance(references.idaes_bt_pr,provenance.Role.oracle_input);
reference translation ideal_inverse(value:MolarEnthalpy,composition:MoleFraction[chemistry.species],members:Set<chemistry.species>)->BtOracleEnthalpy anchors(source=bt_ideal.stock_reference_anchor,target=bt_ideal.oracle_reference_anchor) at(temperature=bt_ideal_oracle.temperature,pressure=bt_ideal_oracle.pressure) provenance(references.idaes_bt_ideal,provenance.Role.oracle_input);
test pr_translations {
let x[j in bt_ideal.aromatics]:MoleFraction=if j==chem.benzene then 0.25{1} else 0.75{1};
let y[j in bt_ideal.aromatics]:MoleFraction=if j==chem.benzene then 0.75{1} else 0.25{1};
let h:MolarEnthalpy=1234{J/mol}; let s:MolarEntropy=10{J/(mol*K)};
let translated_h:PrOracleEnthalpy=bt_pr.stock_to_oracle_enthalpy(h,x,bt_ideal.aromatics);
let translated_s:PrOracleEntropy=bt_pr.stock_to_oracle_entropy(s,x,bt_ideal.aromatics);
expect translated_h==59534{J/mol} tolerance 1e-9{J/mol};
expect bt_pr.stock_to_oracle_enthalpy(h,y,bt_ideal.aromatics)==75934{J/mol} tolerance 1e-9{J/mol};
expect pr_h_inverse(translated_h,x,bt_ideal.aromatics)==h tolerance 1e-9{J/mol};
expect translated_s==-297.89055684058394{J/(mol*K)} tolerance 1e-10{J/(mol*K)};
expect pr_s_inverse(translated_s,x,bt_ideal.aromatics)==s tolerance 1e-10{J/(mol*K)};
expect bt_pr.stock_entropy_anchor(bt_pr_oracle.temperature,100000{Pa},chem.benzene)==0{J/(mol*K)} tolerance 1e-12{J/(mol*K)};
expect bt_pr.pressure_entropy_increment(200000{Pa},100000{Pa})==5.763146321643979{J/(mol*K)} tolerance 1e-11{J/(mol*K)};
expect bt_pr.pressure_entropy_increment(50000{Pa},100000{Pa})==-5.763146321643979{J/(mol*K)} tolerance 1e-11{J/(mol*K)};
expect bt_pr.pressure_entropy_increment(100000{Pa},200000{Pa})==-5.763146321643979{J/(mol*K)} tolerance 1e-11{J/(mol*K)};
}
test ideal_translation {
let x[j in bt_ideal.aromatics]:MoleFraction=if j==chem.benzene then 0.25{1} else 0.75{1};
let y[j in bt_ideal.aromatics]:MoleFraction=if j==chem.benzene then 0.75{1} else 0.25{1};
let oracle:BtOracleEnthalpy=58300{J/mol};
let translated:MolarEnthalpy=bt_ideal.oracle_to_stock(oracle,x,bt_ideal.aromatics);
expect translated==32.375{J/mol} tolerance 1e-9{J/mol};
expect bt_ideal.oracle_to_stock(oracle,y,bt_ideal.aromatics)==-16376.875{J/mol} tolerance 1e-9{J/mol};
expect ideal_inverse(translated,x,bt_ideal.aromatics)==oracle tolerance 1e-9{J/mol};
}
test material_projections {
let component[j in bt_ideal.aromatics]:ComponentFlow=if j==chem.benzene then 3{mol/s} else 7{mol/s};
let flow:Flow=sum(j in bt_ideal.aromatics | component[j]);
let fraction[j in bt_ideal.aromatics]:MoleFraction=component[j]/flow;
let h:MolarEnthalpy=2000{J/mol};
let energy:Power=sum(j in bt_ideal.aromatics | component[j]*h);
let carbon:ElementFlow=sum(j in bt_ideal.aromatics | component[j]*control_volumes.element_coefficient(chemistry.formula[j,ciaaw.C],j,ciaaw.C));
expect flow==10{mol/s} tolerance 1e-12{mol/s};
expect fraction[chem.benzene]==0.3{1} tolerance 1e-12{1};
expect energy==20000{W} tolerance 1e-9{W};
expect carbon==67{mol/s} tolerance 1e-12{mol/s};
}
test stored_energy {
expect reactors.stored_energy(0.1{m^3},10{mol/m^3},2000{J/mol},100000{Pa})==2000{J} tolerance 1e-9{J};
expect reactors.stored_energy(0.1{m^3},10{mol/m^3},2000{J/mol},101325{Pa})==1867.5{J} tolerance 1e-9{J};
expect reactors.stored_energy(0.1{m^3},10{mol/m^3},0{J/mol},101325{Pa})==-132.5{J} tolerance 1e-9{J};
}
test reaction_rates fixture {dof 0;run pure;fix root.T=303.15{K};fix root.concentration[chem.water]=55388{mol/m^3};fix root.concentration[chem.sodium_hydroxide]=100{mol/m^3};fix root.concentration[chem.ethyl_acetate]=100{mol/m^3};fix root.concentration[chem.sodium_acetate]=0{mol/m^3};fix root.concentration[chem.ethanol]=0{mol/m^3};} {
permission kinetic_use families(reaction_forms.reaction_set) allow_unknown true allow_extrapolation false;
child root:reactions.Projection=reactions.Projection(material=saponification.SaponificationReactions.components,selected=saponification.SaponificationReactions.records,selection=saponification.SaponificationReactions.admitted);
let extent[r in saponification.reaction_set]:ReactionExtentRate=0.001{m^3}*root.rate[r];
let source[r in saponification.reaction_set,j in saponification.saponification_species]:ComponentFlow=extent[r]*root.coefficient[r,j];
expect extent[saponification.hydrolysis]==1.221230445517785{mol/s} tolerance 1e-11{mol/s};
expect source[saponification.hydrolysis,chem.ethyl_acetate]==-1.221230445517785{mol/s} tolerance 1e-11{mol/s};
expect source[saponification.hydrolysis,chem.ethanol]==1.221230445517785{mol/s} tolerance 1e-11{mol/s};
expect -extent[saponification.hydrolysis]*root.heat[saponification.hydrolysis]==59840.29183037147{W} tolerance 1e-6{W};
}
fn composition(j:chemistry.species)->Scalar=0.5;
fn ideal_h(T:Temperature,j:chemistry.species)->DeltaH=2000{J/mol};
def InventoryPoint {
child root:vessels.HomogeneousInventory=vessels.HomogeneousInventory(selected=bt_ideal.aromatics,law=helmholtz.ideal,ideal_h=ideal_h,composition=composition,V=0.1{m^3});
}
test CstrPoint {
permission kinetic_use families(reaction_forms.reaction_set) allow_unknown true allow_extrapolation false;
child root:reactors.CSTR=reactors.CSTR(inlet_pkg=saponification.Saponification(defined_state=true),outlet_pkg=saponification.Saponification(defined_state=false),reaction_pkg=saponification.SaponificationReactions);
}
test ExchangerPoint {
child root:heat_exchange.HeatExchanger=heat_exchange.HeatExchanger(hot_inlet_pkg=saponification.Saponification(defined_state=true),hot_outlet_pkg=saponification.Saponification(defined_state=false),cold_inlet_pkg=saponification.Saponification(defined_state=true),cold_outlet_pkg=saponification.Saponification(defined_state=false));
}
test DistributedPoint {
permission kinetic_use families(reaction_forms.reaction_set) allow_unknown true allow_extrapolation false;
child root:reactors.PFR=reactors.PFR(inlet_pkg=saponification.Saponification(defined_state=true),outlet_pkg=saponification.Saponification(defined_state=false),reaction_pkg=saponification.SaponificationReactions,elements=1,use_radau=false);
annotation report root.heat("distributed heat");
}
test ideal_entropy_correction {
let x[j in bt_ideal.aromatics]:MoleFraction=if j==chem.benzene then 0.3{1} else 0.7{1};
// Independent R times log-pressure plus ideal mixing entropy, using R=8.31446261815324.
expect equilibrium.ideal_entropy_increment(200000{Pa},100000{Pa},x,bt_ideal.aromatics)==0.6841379174442977{J/(mol*K)} tolerance 1e-11{J/(mol*K)};
expect equilibrium.ideal_entropy_increment(100000{Pa},100000{Pa},x,bt_ideal.aromatics)==-5.079008404199681{J/(mol*K)} tolerance 1e-11{J/(mol*K)};
expect pressure_fixtures.cp()==29.100619163536336{J/(mol*K)} tolerance 1e-11{J/(mol*K)};
expect pressure_fixtures.molar_s(400{K},chem.nitrogen)==8.371726430596143{J/(mol*K)} tolerance 1e-11{J/(mol*K)};
}

}
"#;
