// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected chemistry admission and projection into ordinary signed source outputs.
use super::composition::{invalid, unsupported};
use super::{ModelDeclaration, PhysicalContext, SourceDeclarations, WorkflowError};
use pse_ids::named_id;
use pse_material::{
    ElementCount, ElementTable, SpeciesId,
    stoichiometry::{Stoichiometry, admit_homogeneous},
};
use pse_relations::{
    columnar::RelationRow,
    generated::{authored::physical_balances as b, enums::*},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate_materials(sources: &SourceDeclarations) -> Result<(), WorkflowError> {
    let c = &sources.composition;
    for system in &c.material_systems {
        if system.species_ids.is_empty()
            || system.phase_ids.is_empty()
            || system.species_ids.iter().collect::<BTreeSet<_>>().len() != system.species_ids.len()
            || system.phase_ids.iter().collect::<BTreeSet<_>>().len() != system.phase_ids.len()
        {
            return Err(invalid(
                [system.material_system_id],
                "material system needs distinct species and phases",
            ));
        }
        for species in &system.species_ids {
            let row = c
                .species
                .iter()
                .find(|s| s.species_id == *species)
                .ok_or_else(|| invalid([*species], "material species declaration missing"))?;
            if row.charge != 0 || row.dissociation_species.is_some() {
                return Err(unsupported(
                    [*species],
                    "ionic/dissociating selected chemistry",
                ));
            }
        }
        for phase in &system.phase_ids {
            let p = c
                .phases
                .iter()
                .find(|p| p.phase_id == *phase)
                .ok_or_else(|| invalid([*phase], "material phase declaration missing"))?;
            for species in &system.species_ids {
                if !c
                    .phase_species
                    .iter()
                    .any(|v| v.phase_id == *phase && v.species_id == *species)
                {
                    return Err(invalid(
                        [*phase, *species],
                        "explicit phase-species membership missing",
                    ));
                }
                let s = c
                    .species
                    .iter()
                    .find(|s| s.species_id == *species)
                    .ok_or_else(|| invalid([*species], "species missing"))?;
                if s.valid_phase_types
                    .as_ref()
                    .is_some_and(|types| !types.contains(&p.phase_type))
                {
                    return Err(invalid(
                        [*species, *phase],
                        "species excludes selected phase type",
                    ));
                }
            }
        }
    }
    for provider in &sources.providers {
        let Some(id) = provider.material_system_id else {
            continue;
        };
        let system = c
            .material_systems
            .iter()
            .find(|s| s.material_system_id == id)
            .ok_or_else(|| invalid([id], "provider material system missing"))?;
        if system.phase_ids.len() != 1
            || provider
                .components
                .iter()
                .map(|s| s.species_id)
                .collect::<BTreeSet<_>>()
                != system.species_ids.iter().copied().collect()
        {
            return Err(invalid(
                [id],
                "homogeneous provider components differ from material system",
            ));
        }
    }
    Ok(())
}

pub(super) fn project(
    model: &mut ModelDeclaration,
    sources: &mut SourceDeclarations,
    physical: &PhysicalContext,
) -> Result<(), WorkflowError> {
    validate_materials(sources)?;
    let c = &sources.composition;
    let elements = if let Some(inventory) = &physical._inventory {
        inventory.elements().clone()
    } else {
        let rows = physical
            .sources
            .iter()
            .find(|(key, _)| key.qualified_name() == "reference.elements")
            .map(|(_, batch)| batch)
            .map(pse_relations::generated::reference::elements::Row::rows)
            .transpose()
            .map_err(super::relation)?
            .unwrap_or_default();
        ElementTable::new(rows.into_iter().map(|r| pse_material::Element {
            id: r.element_id.into(),
            symbol: r.symbol,
            name: r.name,
            atomic_mass: r.atomic_mass,
        }))
        .map_err(|e| invalid([], e.to_string()))?
    };
    let mut compositions: BTreeMap<SpeciesId, Vec<ElementCount>> = BTreeMap::new();
    for item in &c.species_elements {
        compositions
            .entry(item.species_id.into())
            .or_default()
            .push(ElementCount {
                element: item.element_id.into(),
                count: item.count,
            });
    }
    for application in &c.reaction_applications {
        let id = application.application_id;
        if application.provenance.trim().is_empty() {
            return Err(invalid([id], "reaction provenance is required"));
        }
        let system = c
            .material_systems
            .iter()
            .find(|s| s.material_system_id == application.material_system_id)
            .ok_or_else(|| {
                invalid(
                    [id, application.material_system_id],
                    "reaction material system missing",
                )
            })?;
        if system.phase_ids != [application.phase_id] {
            return Err(unsupported(
                [id],
                "selected reaction requires one homogeneous phase",
            ));
        }
        let reaction = c
            .reactions
            .iter()
            .find(|r| r.reaction_id == application.reaction_id)
            .ok_or_else(|| {
                invalid(
                    [id, application.reaction_id],
                    "reaction declaration missing",
                )
            })?;
        if reaction.kind != ReactionKind::Rate
            || reaction.basis != BasisKind::Molar
            || reaction.reaction_phase_id != Some(application.phase_id)
        {
            return Err(unsupported(
                [id, reaction.reaction_id],
                "only explicitly phased molar rate reactions are admitted",
            ));
        }
        if reaction.concentration_form.is_some() {
            return Err(unsupported(
                [reaction.reaction_id],
                "concentration law requires explicit typed authored rate; implicit concentration conversion is unavailable",
            ));
        }
        let terms: Vec<_> = c
            .stoichiometry
            .iter()
            .filter(|s| s.reaction_id == reaction.reaction_id)
            .map(|s| Stoichiometry {
                reaction: s.reaction_id.into(),
                phase: s.phase_id.into(),
                species: s.species_id.into(),
                coefficient: s.coefficient,
            })
            .collect();
        for t in &terms {
            if !system.species_ids.contains(&t.species.as_id()) {
                return Err(invalid(
                    [id, t.species.as_id()],
                    "reaction species outside material system",
                ));
            }
        }
        admit_homogeneous(
            &elements,
            reaction.reaction_id.into(),
            application.phase_id.into(),
            &terms,
            &compositions,
            application.element_tolerance,
        )
        .map_err(|e| invalid([id, reaction.reaction_id], e.to_string()))?;
        let wanted: BTreeSet<_> = terms.iter().map(|t| t.species.as_id()).collect();
        let actual: BTreeSet<_> = application
            .species_balances
            .iter()
            .map(|s| s.species_id)
            .collect();
        if wanted != actual || actual.len() != application.species_balances.len() {
            return Err(invalid(
                [id],
                "every reactive species needs exactly one material balance",
            ));
        }
        let case = model
            .cases
            .iter_mut()
            .find(|c| c.case_id == application.case_id)
            .ok_or_else(|| invalid([id], "reaction case missing"))?;
        let rate = case
            .instances
            .iter()
            .find(|i| i.instance_id == application.rate_instance_id)
            .cloned()
            .ok_or_else(|| invalid([id], "authored rate instance missing"))?;
        if model
            .definitions
            .iter()
            .find(|d| d.definition_id == rate.definition_id)
            .and_then(|d| d.sources.get(application.rate_output as usize))
            .is_none()
        {
            return Err(invalid([id], "authored rate output missing"));
        }
        let mut targets = BTreeSet::new();
        for binding in &application.species_balances {
            if !targets.insert(binding.balance_id) {
                return Err(invalid(
                    [id, binding.balance_id],
                    "reactive species require distinct balance owners",
                ));
            }
            let coefficient = terms
                .iter()
                .find(|s| s.species.as_id() == binding.species_id)
                .ok_or_else(|| invalid([id], "stoichiometry binding"))?
                .coefficient;
            let balance = sources
                .balances
                .iter_mut()
                .find(|b| b.balance_id == binding.balance_id && b.case_id == application.case_id)
                .ok_or_else(|| invalid([id, binding.balance_id], "material balance missing"))?;
            let quantity = physical
                .quantities
                .quantity_type(balance.quantity_id.into())
                .map_err(super::math)?;
            let dims = physical
                .quantities
                .kind(quantity.key.kind)
                .map_err(super::math)?
                .dimension;
            if !dims
                .exponents()
                .iter()
                .zip([0, 0, -1, 0, 1, 0, 0, 0])
                .all(|(v, e)| v.num() == e && v.den() == 1)
            {
                return Err(invalid(
                    [id, balance.balance_id],
                    "reaction extent and species rates require molar amount per time",
                ));
            }
            let generated = named_id(id, &binding.species_id.to_hex());
            balance
                .terms
                .push(b::AuthoredPhysicalBalancesFieldTermsItem {
                    source_id: generated,
                    role: if coefficient > 0.0 {
                        BalanceRole::Generation
                    } else {
                        BalanceRole::Consumption
                    },
                    multiplier: coefficient.abs(),
                    mode: None,
                    transfer_id: None,
                    instance_id: rate.instance_id,
                    output: application.rate_output,
                });
        }
        let heat = case
            .instances
            .iter()
            .find(|i| i.instance_id == application.heat_instance_id)
            .ok_or_else(|| invalid([id], "explicit heat model instance missing"))?;
        if model
            .definitions
            .iter()
            .find(|d| d.definition_id == heat.definition_id)
            .and_then(|d| d.sources.get(application.heat_output as usize))
            .is_none()
        {
            return Err(invalid([id], "explicit heat model output missing"));
        }
        let energy = sources
            .balances
            .iter_mut()
            .find(|b| {
                b.balance_id == application.energy_balance_id && b.case_id == application.case_id
            })
            .ok_or_else(|| invalid([id], "reaction energy balance missing"))?;
        let quantity = physical
            .quantities
            .quantity_type(energy.quantity_id.into())
            .map_err(super::math)?;
        let dims = physical
            .quantities
            .kind(quantity.key.kind)
            .map_err(super::math)?
            .dimension;
        if !dims
            .exponents()
            .iter()
            .zip([2, 1, -3, 0, 0, 0, 0, 0])
            .all(|(v, e)| v.num() == e && v.den() == 1)
        {
            return Err(invalid(
                [id, energy.balance_id],
                "explicit reaction heat must be an energy rate",
            ));
        }
        energy
            .terms
            .push(b::AuthoredPhysicalBalancesFieldTermsItem {
                source_id: named_id(id, "heat"),
                role: BalanceRole::HeatIn,
                multiplier: 1.0,
                transfer_id: None,
                mode: None,
                instance_id: heat.instance_id,
                output: application.heat_output,
            });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{
        ModelBuilder,
        tests::{id, physical, runtime},
    };
    use pse_quantity::QuantityAdditionKind;
    use pse_quantity::*;
    use pse_relations::generated::{
        authored::{computation_models as m, physical_balances as b},
        reference::elements,
    };
    use std::sync::{Arc, atomic::AtomicBool};
    fn fixture() -> ModelBuilder {
        let mut physical = physical();
        let mut q = physical.quantities.to_builder();
        let neutral = physical
            .quantities
            .quantity_type(standard::ids::quantity("neutral"))
            .unwrap();
        for (n, dims) in [
            (110, [0, 0, -1, 0, 1, 0, 0, 0]),
            (111, [2, 1, -3, 0, 0, 0, 0, 0]),
        ] {
            let dimension = DimensionVector::new(dims.map(|e| Ratio::new(e, 1).unwrap()));
            q.kind(QuantityKind {
                id: id(n).into(),
                dimension,
                extensive: true,
                addition_kind: QuantityAdditionKind::Additive,
            });
            q.unit(Unit {
                id: id(n).into(),
                symbol: format!("reaction-test-{n}"),
                dimension,
                scale_to_canonical: 1.,
                offset_to_canonical: 0.,
                is_affine: false,
                reference_state: None,
            });
            let mut qty = neutral.clone();
            qty.id = id(n).into();
            qty.key.kind = id(n).into();
            qty.canonical_unit = id(n).into();
            q.quantity_type(qty);
        }
        physical.quantities = Arc::new(q.build().unwrap());
        physical.key = pse_compiler::workspace::physical_identity(
            &physical.quantities,
            &physical.preconditions,
        );
        let registry = pse_schema::registry().unwrap();
        let mut elements = elements::Builder::with_registry(registry, 2).unwrap();
        for (n, symbol, mass) in [(120, "C", 0.012011), (121, "H", 0.001008)] {
            elements
                .push(elements::Row {
                    element_id: id(n),
                    symbol: symbol.into(),
                    name: symbol.into(),
                    atomic_mass: mass,
                })
                .unwrap();
        }
        physical.sources.insert(
            registry.relation("reference.elements").unwrap().key,
            elements.finish().unwrap(),
        );
        let mut model:ModelDeclaration=serde_json::from_value(serde_json::json!({"model_id":id(1),"name":"constructed balanced reaction","definitions":[],"domains":[],"groups":[],"cases":[{"case_id":id(2),"name":"selected","variables":[],"parameters":[],"instances":[],"rows":[],"objective":null,"values":[]}]})).unwrap();
        let case = &mut model.cases[0];
        for (n, quantity, value) in [
            (10, 110, 1.0),
            (11, 111, 5.0),
            (12, 110, 1.0),
            (13, 110, 1.0),
            (14, 110, 2.0),
            (15, 111, 5.0),
        ] {
            case.parameters
                .push(m::AuthoredComputationModelsFieldCasesItemParametersItem {
                    symbol_id: id(n),
                    quantity_id: id(quantity),
                    unit_id: id(quantity),
                });
            case.values
                .push(m::AuthoredComputationModelsFieldCasesItemValuesItem {
                    symbol_id: id(n),
                    value,
                });
            model.definitions.push(serde_json::from_value(serde_json::json!({"definition_id":id(n+20),"sources":["v"],"formals":[{"path":"v","quantity_id":id(quantity)}],"domains":[],"groups":[],"providers":[],"units":[],"literals":[]})).unwrap());
            case.instances.push(serde_json::from_value(serde_json::json!({"instance_id":id(n+40),"definition_id":id(n+20),"slots":[{"source_id":id(n),"formal_quantity_id":id(quantity),"formal_unit_id":id(quantity)}],"contributions":[]})).unwrap());
        }
        let mut builder = ModelBuilder::from_declaration(runtime(), model, physical);
        let c = &mut builder.sources.composition;
        c.material_systems.push(serde_json::from_value(serde_json::json!({"material_system_id":id(70),"package_id":id(71),"name":"alkanes","species_ids":[id(80),id(81),id(82)],"phase_ids":[id(72)],"doc":""})).unwrap());
        c.phases.push(serde_json::from_value(serde_json::json!({"phase_id":id(72),"package_id":id(71),"name":"vapor","phase_type":"vaporPhase","is_solvent_phase":false,"doc":""})).unwrap());
        for (n, formula, carbon, hydrogen) in [
            (80, "CH4", 1., 4.),
            (81, "C3H8", 3., 8.),
            (82, "C2H6", 2., 6.),
        ] {
            c.species.push(serde_json::from_value(serde_json::json!({"species_id":id(n),"package_id":id(71),"name":formula,"formula":formula,"mw":null,"component_type":"Component","charge":0,"dissociation_species":null,"valid_phase_types":["vaporPhase"],"doc":""})).unwrap());
            c.phase_species.push(
                serde_json::from_value(serde_json::json!({"phase_id":id(72),"species_id":id(n)}))
                    .unwrap(),
            );
            for (element, count) in [(120, carbon), (121, hydrogen)] {
                c.species_elements.push(serde_json::from_value(serde_json::json!({"species_id":id(n),"element_id":id(element),"count":count})).unwrap());
            }
            c.stoichiometry.push(serde_json::from_value(serde_json::json!({"reaction_id":id(73),"phase_id":id(72),"species_id":id(n),"coefficient":if n==82 {2.} else {-1.}})).unwrap());
        }
        c.reactions.push(serde_json::from_value(serde_json::json!({"reaction_id":id(73),"package_id":id(71),"name":"constructed alkane conservation control","kind":"rate","basis":"molar","concentration_form":null,"reaction_phase_id":id(72),"doc":"Not empirical kinetics"})).unwrap());
        c.reaction_applications.push(serde_json::from_value(serde_json::json!({"application_id":id(74),"model_id":id(1),"case_id":id(2),"material_system_id":id(70),"reaction_id":id(73),"phase_id":id(72),"rate_instance_id":id(50),"rate_output":0,"species_balances":[{"species_id":id(80),"balance_id":id(90)},{"species_id":id(81),"balance_id":id(91)},{"species_id":id(82),"balance_id":id(92)}],"energy_balance_id":id(93),"heat_instance_id":id(51),"heat_output":0,"element_tolerance":0.0,"provenance":"constructed stoichiometric and explicit heat control"})).unwrap());
        for (balance, instance, qty, role) in [
            (90, 52, 110, BalanceRole::Inlet),
            (91, 53, 110, BalanceRole::Inlet),
            (92, 54, 110, BalanceRole::Outlet),
            (93, 55, 111, BalanceRole::HeatOut),
        ] {
            builder.balance(b::Row {
                balance_id: id(balance),
                model_id: id(1),
                case_id: id(2),
                quantity_id: id(qty),
                accumulation: None,
                tolerance: 1e-8,
                integral_tolerance: None,
                provenance: "declared species/energy flux control".into(),
                terms: vec![b::AuthoredPhysicalBalancesFieldTermsItem {
                    multiplier: 1.0,
                    source_id: id(instance),
                    role,
                    transfer_id: None,
                    mode: None,
                    instance_id: id(instance),
                    output: 0,
                }],
                impulses: vec![],
            });
        }
        builder
    }
    #[test]
    fn selected_reaction_closes_material_elements_and_explicit_heat() {
        let revision = fixture().freeze().unwrap();
        let input = revision.0.cases[&id(2)].as_ref().clone();
        let mut compiler =
            pse_compiler::workspace::CompilerWorkspace::new(input.clone(), Default::default())
                .unwrap();
        let plan = compiler.admit_selected_case(id(2)).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let assembly = Arc::new(
            plan.compile(Default::default(), Default::default(), &cancel)
                .unwrap(),
        );
        let mut worker = assembly.worker(BTreeMap::new(), cancel);
        let values = pse_math::binding::CaseValues {
            scalars: input.values.clone(),
        };
        let residuals = worker.constraints(&values).unwrap();
        assert_eq!(residuals, vec![0.; 4]);
        let source_values = worker.constraint_sources().unwrap();
        for balance in revision.resolved_balances() {
            assert_eq!(
                super::super::balances::closure(balance, &source_values).unwrap(),
                0.0
            );
        }
        assert_eq!(revision.source_declarations().balances[0].terms.len(), 1);
        assert_eq!(revision.resolved_balances()[0].terms.len(), 2);
        let mut tolerance = revision.edit();
        tolerance.sources.composition.reaction_applications[0].element_tolerance = 1e-10;
        let tolerance = tolerance.freeze().unwrap();
        assert_ne!(revision.identity(), tolerance.identity());
        assert_eq!(
            revision.case_identity(id(2)),
            tolerance.case_identity(id(2))
        );
        let mut prose = revision.edit();
        prose.sources.composition.species[0].doc = "clarified documentation".into();
        prose.sources.composition.reactions[0].name = "editorial name".into();
        assert_eq!(revision.identity(), prose.freeze().unwrap().identity());
        let mut changed = revision.edit();
        changed.sources.composition.stoichiometry[0].coefficient = -2.;
        assert!(matches!(changed.freeze(), Err(WorkflowError::Boundary(_))));
        let mut no_heat = revision.edit();
        no_heat.sources.composition.reaction_applications[0].heat_instance_id = id(250);
        assert!(no_heat.freeze().unwrap_err().to_string().contains("heat"));
        let mut unrelated = revision.edit();
        unrelated.sources.composition.reaction_applications[0].case_id = id(250);
        unrelated.sources.composition.stoichiometry[0].coefficient = -2.;
        assert!(unrelated.freeze().is_ok());
    }
}
