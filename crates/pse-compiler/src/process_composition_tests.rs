// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Focused structural controls over the original indexed process owners.
#![allow(
    clippy::unwrap_used,
    reason = "focused composition contract assertions"
)]
use crate::{
    authored_transfer_tests::{context, reference_sources, root, rows},
    workspace::{CompilerWorkspace, WorkspaceLimits},
};
use pse_modeling::{Bindings, Limits, PhysicalScope, specialize::root_instance};
const COMPOSE: &str = r#"package process_composition {
use chemistry @"1.0.0";use chem @"1.0.0";use kinds @"1.0.0";
use mixing @"1.0.0";use separation @"1.0.0";use reactors @"1.0.0";use saponification @"1.0.0";
use distributed_volumes @"1.0.0";use correlations @"1.0.0";use properties @"1.0.0";use compatibility @"1.0.0";use references @"1.0.0";
entity kinds.port_set first {} entity kinds.port_set second {}
test ThreeUnits {
permission solvent_use records(correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit],correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit]) allow_unknown true allow_extrapolation false;
domain t:Time from 0{s} to 1{s};
when analysis.route==analysis.steady {discretize grid on t using stationary(elements=1,order=1);}
when analysis.route==analysis.integrated {discretize grid on t using integrated(elements=1,order=1);}
when analysis.route==analysis.simultaneous {discretize grid on t using distributed_volumes.radau(elements=1,order=2);}
child mixer[i in t]:mixing.Mixer=mixing.Mixer(inlet_pkg=saponification.Saponification(defined_state=true),outlet_pkg=saponification.Saponification(defined_state=false),inlets={first,second});
child reactor[i in t]:reactors.CSTR=reactors.CSTR(inlet_pkg=saponification.Saponification(defined_state=true),outlet_pkg=saponification.Saponification(defined_state=false),reaction_pkg=saponification.SaponificationReactions,holdup=true,times={i});
child separator[i in t]:separation.Separator=separation.Separator(pkg=saponification.Saponification(defined_state=true),outlets={first,second},component_enthalpy=saponification.component_enthalpy);
connect mixed: [i in t] mixer[i].outlet_port -> reactor[i].inlet_port;
connect reacted: [i in t] reactor[i].outlet_port -> separator[i].inlet_port;
when analysis.dynamic {
eq initial_inventory[j in {chem.sodium_hydroxide,chem.ethyl_acetate,chem.sodium_acetate,chem.ethanol}]:reactor[0{s}].phase_material_holdup[chemistry.liquid,j]==1{mol};
eq initial_energy:reactor[0{s}].phase_energy_holdup[chemistry.liquid]==1000{J};
}
}
}"#;
#[test]
fn indexed_process_composition_keeps_memoryless_units_algebraic() {
    let text = format!(
        "{}\n{}\n{COMPOSE}",
        reference_sources(),
        include_str!("../../../packages/reference/process/models/mixer.pse")
    );
    let declarations = rows(&text);
    let id = root(&declarations, "process_composition", "ThreeUnits");
    let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    for route in [
        pse_modeling::analysis::Route::Steady,
        pse_modeling::analysis::Route::Integrated,
        pse_modeling::analysis::Route::Simultaneous,
    ] {
        let bindings = Bindings::default().with_analysis(route);
        let model = workspace
            .specialize_modeling(id, root_instance(id), bindings, Limits::default())
            .unwrap();
        assert!(model.connections.values().all(|c| c.bindings.len() == 7));
        assert_eq!(
            model.inventory_balances.len(),
            if route == pse_modeling::analysis::Route::Steady {
                0
            } else {
                5
            }
        );
        assert_eq!(
            model.derivatives.len(),
            if route == pse_modeling::analysis::Route::Integrated {
                5
            } else {
                0
            }
        );
        assert!(model.derivatives.values().all(|d| {
            model.instances[&d.lineage.instance]
                .path
                .contains("reactor")
        }));
        assert!(model.closures.values().any(|c| {
            c.mode == pse_model::generated::enums::ModelingAccumulatorMode::Observation
                && model.instances[&c.lineage.instance]
                    .path
                    .contains("separator")
        }));
    }
}

#[test]
fn authored_vessel_temporal_closures_use_original_inventory_and_flux() {
    use crate::workspace::ModelingOutput;
    use pse_kernels::DerivativeOrder;
    use pse_math::{
        assembly::AssemblyLimits, binding::CaseValues, jets::EvaluationLimits,
        library::Optimization,
    };
    use std::{
        collections::{BTreeMap, BTreeSet},
        sync::{Arc, atomic::AtomicBool},
    };
    let source = format!(
        r#"{}
package conserved_vessel {{
use chemistry @"1.0.0";use bt_ideal @"1.0.0";use vessels @"1.0.0";use helmholtz @"1.0.0";
fn composition(j:chemistry.species)->Scalar=0.5;
fn ideal_h(T:Temperature,j:chemistry.species)->DeltaH=2000{{J/mol}};
def Root {{child root:vessels.Vessel=vessels.Vessel(selected=bt_ideal.aromatics,law=helmholtz.ideal,ideal_h=ideal_h,composition=composition);}}
}}"#,
        reference_sources()
    );
    let declarations = rows(&source);
    let id = root(&declarations, "conserved_vessel", "Root");
    let inputs = context();
    let quantities = inputs.quantities.clone();
    let mut workspace = CompilerWorkspace::new(inputs, WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let integrated = workspace
        .specialize_modeling(
            id,
            root_instance(id),
            Bindings::default().with_analysis(pse_modeling::analysis::Route::Integrated),
            Limits::default(),
        )
        .unwrap();
    assert_eq!(integrated.inventory_balances.len(), 2);
    assert_eq!(integrated.derivatives.len(), 2);
    assert!(
        integrated
            .inventory_balances
            .values()
            .all(|balance| balance.state.is_some()
                && integrated.integrals.contains_key(&balance.flux_id))
    );
    let mut bindings =
        Bindings::default().with_analysis(pse_modeling::analysis::Route::Simultaneous);
    bindings.demand = [
        "root.cell[0{s}].amount",
        "root.cell[0{s}].energy",
        "root.cell[1{s}].amount",
        "root.cell[1{s}].energy",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let cancel = Arc::new(AtomicBool::new(false));
    let prepared = workspace
        .prepare_modeling_cancellable(
            id,
            root_instance(id),
            bindings,
            Limits::default(),
            cancel.clone(),
        )
        .unwrap();
    assert_eq!(prepared.model.inventory_balances.len(), 2);
    assert!(prepared.model.derivatives.is_empty());
    let subjects = prepared
        .model
        .inventory_balances
        .keys()
        .map(|id| pse_ids::named_id(*id, "temporal-closure"))
        .collect::<BTreeSet<_>>();
    assert_eq!(subjects.len(), 2);
    assert!(
        subjects
            .iter()
            .all(|id| prepared.model.closures[id].observation_only)
    );
    let plan = prepared
        .admitted
        .plan(
            &quantities,
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
    let mut worker = assembly.worker(BTreeMap::new(), cancel);
    // A 0.1 J energy drift is within the authored 100 J engineering allowance.
    // Exercise a real refusal with 200 J, while the molar drift exceeds 0.001 mol.
    for (amount_drift, energy_drift, closes) in [(0.0, 0.0, true), (0.1, 200.0, false)] {
        let mut values = CaseValues {
            scalars: prepared
                .admitted
                .inputs
                .iter()
                .map(|id| {
                    let symbol = &prepared.model.symbols[id];
                    let value = match &symbol.initial {
                        Some(pse_modeling::specialize::Value::Number { bits, .. }) => {
                            f64::from_bits(*bits)
                        }
                        _ if symbol.lineage.path.ends_with(".phase.T") => 300.0,
                        _ if symbol.lineage.path.ends_with(".phase.rho") => 10.0,
                        _ if symbol.lineage.path.contains(".phase.amount") => 0.5,
                        _ => 1.0,
                    };
                    (*id, value)
                })
                .collect(),
        };
        for (path, value) in [
            ("root.cell[0{s}].amount", 3.456556634933607),
            ("root.cell[1{s}].amount", 3.456556634933607 + amount_drift),
            ("root.cell[0{s}].energy", 1202.3168203660216),
            ("root.cell[1{s}].energy", 1212.3168203660216 + energy_drift),
        ] {
            values.scalars.insert(prepared.model.paths[path], value);
        }
        let computed = worker.constraints(&values).unwrap();
        let ordered = prepared.admitted.ordered_values(&computed).unwrap();
        let contributions = prepared
            .admitted
            .outputs
            .iter()
            .zip(ordered)
            .filter_map(|(output, value)| match output {
                ModelingOutput::Contribution {
                    accumulator,
                    contribution,
                } if subjects.contains(accumulator) => Some((*contribution, value)),
                _ => None,
            })
            .collect();
        let assessed = prepared
            .model
            .assess_closures(&contributions, &subjects)
            .unwrap();
        assert_eq!(assessed.len(), 2);
        assert!(
            assessed
                .iter()
                .all(|closure| closure.satisfied == Some(closes)),
            "{assessed:?}"
        );
    }
}

#[test]
fn indexed_balance_operations_keep_all_applicable_material_bases() {
    let bases = [
        ("componentTotal", "material", 5),
        ("componentPhase", "phase_material", 5),
        ("elementTotal", "element_material", 4),
        ("total", "total_material", 1),
    ];
    let mut definitions = String::new();
    for (basis, _, _) in bases {
        definitions.push_str(&format!(r#"
def CV_{basis}(times:Set<Time>={{}},holdup:Boolean=false):control_volumes.ControlVolume0D {{
override param inlet_package:equilibrium.ThermoPackage=saponification.Saponification(defined_state=true);
override param outlet_package:equilibrium.ThermoPackage=saponification.Saponification(defined_state=false);
override param material_basis:control_volumes.MaterialBalance=control_volumes.MaterialBalance.{basis};
override param elements:Set<chemistry.element>={{ciaaw.C,ciaaw.H,ciaaw.O,ciaaw.Na}};
override param time:Set<Time>=times;
override param has_holdup:Boolean=holdup;
}}
"#));
        for stored in [false, true] {
            if stored && !matches!(basis, "elementTotal" | "total") {
                continue;
            }
            let suffix = if stored { "stored" } else { "memoryless" };
            let child = if stored {
                format!(
                    "domain t:Time from 0{{s}} to 1{{s}};discretize grid on t using integrated(elements=1,order=1);child cv[i in t]:CV_{basis}=CV_{basis}(times={{i}},holdup=true);"
                )
            } else {
                format!("child cv:CV_{basis}=CV_{basis}();")
            };
            definitions.push_str(&format!(r#"
test Root_{basis}_{suffix} {{
permission solvent_use records(correlations.constant[chem.water,properties.heat_capacity,compatibility.PhaseType.liquidPhase,references.saponification_caloric_fit],correlations.constant_density[chem.water,properties.liquid_molar_density,compatibility.PhaseType.liquidPhase,references.saponification_density_fit]) allow_unknown true allow_extrapolation false;
{child}
}}
"#));
        }
    }
    let source = format!(
        r#"{}
{}
package indexed_basis {{
use chemistry @"1.0.0";use ciaaw @"1.0.0";use control_volumes @"1.0.0";
use equilibrium @"1.0.0";use kinds @"1.0.0";use mixing @"1.0.0";use separation @"1.0.0";use saponification @"1.0.0";
use correlations @"1.0.0";use chem @"1.0.0";use properties @"1.0.0";use compatibility @"1.0.0";use references @"1.0.0";
entity kinds.port_set first {{}}entity kinds.port_set second {{}}
{definitions}
}}"#,
        reference_sources(),
        include_str!("../../../packages/reference/process/models/mixer.pse")
    );
    let declarations = rows(&source);
    let ids = bases.map(|(basis, _, _)| {
        root(
            &declarations,
            "indexed_basis",
            &format!("Root_{basis}_memoryless"),
        )
    });
    let stored_ids = ["elementTotal", "total"].map(|basis| {
        root(
            &declarations,
            "indexed_basis",
            &format!("Root_{basis}_stored"),
        )
    });
    let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    for ((basis, family, count), id) in bases.into_iter().zip(ids) {
        for route in [
            pse_modeling::analysis::Route::Steady,
            pse_modeling::analysis::Route::Integrated,
        ] {
            let model = workspace
                .specialize_modeling(
                    id,
                    root_instance(id),
                    Bindings::default().with_analysis(route),
                    Limits::default(),
                )
                .unwrap();
            assert_eq!(
                model
                    .closures
                    .values()
                    .filter(|c| c.lineage.path.ends_with(&format!(".{family}")))
                    .count(),
                count,
                "{basis}"
            );
            assert!(
                model.derivatives.is_empty() && model.inventory_balances.is_empty(),
                "memoryless {basis}"
            );
        }
    }
    for (basis, id) in ["elementTotal", "total"].into_iter().zip(stored_ids) {
        let error = workspace
            .specialize_modeling(
                id,
                root_instance(id),
                Bindings::default().with_analysis(pse_modeling::analysis::Route::Integrated),
                Limits::default(),
            )
            .unwrap_err();
        assert!(
            error.to_string().contains("explicit inventory projection"),
            "{basis}: {error}"
        );
    }
}
