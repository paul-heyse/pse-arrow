// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared source closure for bounded actual-reference compiler/runtime controls.
use pse_authoring::{
    ParseBudget,
    language::{Declaration, IdentityPolicy, parse},
};
use pse_ids::SemanticId;
use std::collections::BTreeMap;

/// Read current source documents, including the process consumers and both caloric bindings.
/// External data transport occurrences are outside this selected declaration control; numerical
/// cases supply small synthetic keyed parameter rows for the selected declarations. Full package/data transport is
/// qualified in Plan 25k. No source function or process equation is copied into this test.
pub(crate) fn reference_sources() -> String {
    // The retained seed selection below has fixture data lineage to the synthetic
    // pressure rows. Admit that data-only dependency without changing its science.
    let sources = SOURCES.join("\n").replacen(
        "package bt_ideal {",
        "package bt_ideal { use transfer_parameters @\"1.0.0\";",
        1,
    );
    format!("{sources}\n{SYNTHETIC_PARAMETERS}")
}

// Complete source documents for the changed packages and their imported dependency closure.
// Keep this explicit: adding an unrelated reference package does not broaden these controls.
const SOURCES: &[&str] = &[
    include_str!("../../packages/reference/domain/models/numerical-policy.pse"),
    include_str!("../../packages/reference/thermodynamics/models/aqueous.pse"),
    include_str!("../../packages/reference/seed-data/models/bt-ideal.pse"),
    include_str!("../../packages/reference/seed-data/models/bt-pr.pse"),
    include_str!("../../packages/reference/methods/models/caloric.pse"),
    include_str!("../../packages/reference/seed-data/models/caloric.pse"),
    include_str!("../../packages/reference/data/species/models/catalogue.pse"),
    include_str!("../../packages/reference/physical/models/chemistry.pse"),
    include_str!("../../packages/reference/data/ciaaw/models/catalogue.pse"),
    include_str!("../../packages/reference/physical/models/compatibility.pse"),
    include_str!("../../packages/reference/domain/models/constants.pse"),
    include_str!("../../packages/reference/process/models/control-volumes.pse"),
    include_str!("../../packages/reference/process/models/separator.pse"),
    include_str!("../../packages/reference/methods/models/correlations.pse"),
    include_str!("../../packages/reference/methods/models/cubic.pse"),
    include_str!("../../packages/reference/process/models/control-volume-1d.pse"),
    include_str!("../../packages/reference/seed-data/models/eos-data.pse"),
    include_str!("../../packages/reference/thermodynamics/models/equilibrium.pse"),
    include_str!("../../packages/reference/process/models/heat-exchanger.pse"),
    include_str!("../../packages/reference/process/models/homogeneous-units.pse"),
    include_str!("../../packages/reference/thermodynamics/models/phase-stability.pse"),
    include_str!("../../packages/reference/thermodynamics/models/helmholtz.pse"),
    include_str!("../../packages/reference/thermodynamics/models/ideal-caloric-state.pse"),
    include_str!("../../packages/reference/process/models/pressure-changer.pse"),
    include_str!("../../packages/reference/seed-data/models/pressure-fixtures.pse"),
    include_str!("../../packages/reference/data/oracles/idaes-2.13/models/parameters.pse"),
    include_str!("../../packages/reference/domain/models/interactions.pse"),
    include_str!("../../packages/reference/physical/models/kinds.pse"),
    include_str!("../../packages/reference/physical/models/math.pse"),
    include_str!("../../packages/reference/thermodynamics/models/nested-equilibrium.pse"),
    include_str!("../../packages/reference/thermodynamics/models/peng-robinson.pse"),
    include_str!("../../packages/reference/seed-data/models/phase-data.pse"),
    include_str!("../../packages/reference/thermodynamics/models/pr-equilibrium.pse"),
    include_str!("../../packages/reference/domain/models/properties.pse"),
    include_str!("../../packages/reference/domain/models/provenance.pse"),
    include_str!("../../packages/reference/methods/models/pure-properties.pse"),
    include_str!("../../packages/reference/methods/models/reaction-forms.pse"),
    include_str!("../../packages/reference/thermodynamics/models/reactions.pse"),
    include_str!("../../packages/reference/process/models/reactors.pse"),
    include_str!("../../packages/reference/data/references/models/references.pse"),
    include_str!("../../packages/reference/seed-data/models/saponification.pse"),
    include_str!("../../packages/reference/process/models/delta-t.pse"),
    include_str!("../../packages/reference/process/models/units.pse"),
    include_str!("../../packages/reference/process/models/vessels.pse"),
];

pub(crate) fn rows(text: &str) -> Vec<Declaration> {
    let mut parsed = parse(
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
        .map(|row| (row.declaration_id, row.name.clone()))
        .collect::<BTreeMap<_, _>>();
    // Retain the actual seed's pressure choice and its completeness/coherence
    // contract. Only its test data lineage changes: the selected synthetic rows
    // replace the external parameter transport for this bounded control.
    for row in &mut parsed {
        if row.name == "pressure_bindings"
            && row
                .parent_id
                .and_then(|id| packages.get(&id))
                .is_some_and(|p| p == "bt_ideal")
            && let Some(dataset) = &mut row.value.dataset
        {
            for lineage in &mut dataset.provenance.lineage {
                if lineage.path == ["idaes_thermo", "rpp4_pressure_oracles"] {
                    lineage.path = vec!["transfer_parameters".into(), "pressure".into()];
                }
            }
        }
    }
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
// Synthetic liquid, pressure and critical rows satisfy the original seed selection
// in bounded structural/admission controls. Their coefficients do not qualify an
// oracle or coupled scientific journey; Plan 25k uses qualified original inputs.
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
