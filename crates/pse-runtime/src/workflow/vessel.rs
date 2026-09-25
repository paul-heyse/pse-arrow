// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed vessel construction over generated declarations and library mathematics.
use super::{ModelBuilder, NativeProviderDeclaration, WorkflowError, contract};
use pse_ids::{SemanticId, named_id};
use pse_kernels::Port;
use pse_quantity::QuantityTypeId;
use pse_relations::generated::{
    authored::{computation_models as m, dynamic_cases as d, physical_balances as b},
    enums::{BalanceRole, NativeVariableDomain},
};
use std::collections::BTreeMap;
/// Complete physical vessel roles; absent optional valve means prescribed outflow.
#[derive(Clone, Debug)]
pub struct VesselPorts {
    /// Physical n port.
    pub n: Port,
    /// Physical u port.
    pub u: Port,
    /// Physical temperature port.
    pub temperature: Port,
    /// Physical density port.
    pub density: Port,
    /// Physical pressure port.
    pub pressure: Port,
    /// Physical time port.
    pub time: Port,
    /// Physical volume port.
    pub volume: Port,
    /// Physical methane port.
    pub methane: Port,
    /// Physical ethane port.
    pub ethane: Port,
    /// Physical inflow port.
    pub inflow: Port,
    /// Physical outflow port.
    pub outflow: Port,
    /// Physical inlet enthalpy port.
    pub inlet_enthalpy: Port,
    /// Physical heat port.
    pub heat: Port,
    /// Physical n0 port.
    pub n0: Port,
    /// Physical u0 port.
    pub u0: Port,
    /// Physical temperature0 port.
    pub temperature0: Port,
    /// Physical density0 port.
    pub density0: Port,
    /// Physical pressure0 port.
    pub pressure0: Port,
    /// Valve coefficient, downstream pressure and explicit positive pressure transition width.
    pub valve: Option<(Port, Port, Port)>,
}
/// Declared physical quantities of generated rates and algebraic closures.
#[derive(Clone, Debug)]
pub struct VesselQuantities {
    /// Conserved amount per time.
    pub amount_rate: QuantityTypeId,
    /// Conserved energy per time.
    pub energy_rate: QuantityTypeId,
    /// Amount closure.
    pub amount: QuantityTypeId,
    /// Internal energy closure.
    pub energy: QuantityTypeId,
    /// Pressure closure.
    pub pressure: QuantityTypeId,
}
/// Inputs to a generated fixed-composition homogeneous vessel.
#[derive(Clone, Debug)]
pub struct VesselRecipe {
    /// Identity namespace.
    pub id: SemanticId,
    /// Complete typed physical roles.
    pub ports: VesselPorts,
    /// Initial/fixed values in source units.
    pub values: BTreeMap<SemanticId, f64>,
    /// Physical rate and closure quantities.
    pub functions: VesselQuantities,
    /// Canonical state scales in n/u/T/rho/P order.
    pub state_scales: [f64; 5],
    /// Algebraic amount/energy/pressure residual scales.
    pub residual_scales: [f64; 3],
    /// Amount and energy absolute conservation tolerances in canonical units.
    pub balance_tolerances: [f64; 2],
    /// Native FeOS factory and explicit operating envelope.
    pub provider: NativeProviderDeclaration,
}
impl VesselRecipe {
    /// Produce inspectable authored equations, contributions and dynamics before admission.
    /// These ordinary declarations may be edited or serialized and enter the common freeze boundary.
    /// # Errors
    /// Missing physical roles, values or invalid declared scales/tolerances.
    pub fn declarations(
        self,
        model: SemanticId,
        physical: &super::PhysicalContext,
    ) -> Result<(super::ModelDeclaration, super::SourceDeclarations), WorkflowError> {
        let recipe = self;
        let mut row = super::ModelDeclaration {
            model_id: model,
            name: "homogeneous vessel".into(),
            definitions: vec![],
            domains: vec![],
            groups: vec![],
            cases: vec![],
        };
        let mut sources = super::SourceDeclarations::default();
        let state_names = ["n", "u", "temperature", "density", "pressure"];
        let mut roles = vec![
            ("n", &recipe.ports.n),
            ("u", &recipe.ports.u),
            ("temperature", &recipe.ports.temperature),
            ("density", &recipe.ports.density),
            ("pressure", &recipe.ports.pressure),
            ("time", &recipe.ports.time),
            ("volume", &recipe.ports.volume),
            ("methane", &recipe.ports.methane),
            ("ethane", &recipe.ports.ethane),
            ("inflow", &recipe.ports.inflow),
            ("outflow", &recipe.ports.outflow),
            ("inlet_enthalpy", &recipe.ports.inlet_enthalpy),
            ("heat", &recipe.ports.heat),
            ("n0", &recipe.ports.n0),
            ("u0", &recipe.ports.u0),
            ("temperature0", &recipe.ports.temperature0),
            ("density0", &recipe.ports.density0),
            ("pressure0", &recipe.ports.pressure0),
        ];
        if let Some((k, p, width)) = &recipe.ports.valve {
            roles.extend([("valve_k", k), ("downstream", p), ("valve_width", width)]);
        }
        if recipe.values.len() != roles.len()
            || roles
                .iter()
                .any(|(_, p)| recipe.values.get(&p.id).is_none_or(|v| !v.is_finite()))
            || recipe
                .state_scales
                .iter()
                .chain(&recipe.residual_scales)
                .chain(&recipe.balance_tolerances)
                .any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err(contract(
                "vessel values or positive scale/tolerance contract",
            ));
        }
        let name = |s: &str| named_id(recipe.id, s);
        let mut pressure = recipe.provider.clone();
        pressure.model_id = model;
        pressure.name = format!("vessel_pressure_{}", recipe.id.to_hex());
        pressure.output = 0;
        let mut enthalpy = pressure.clone();
        enthalpy.name = format!("vessel_enthalpy_{}", recipe.id.to_hex());
        enthalpy.output = 1;
        let coordinates = composition_coordinates(&pressure)?;
        let pe = format!(
            "kernel.{}(temperature,density,{coordinates})",
            pressure.name
        );
        let he = format!(
            "kernel.{}(temperature,density,{coordinates})",
            enthalpy.name
        );
        let valve_name = format!("vessel_valve_{}", recipe.id.to_hex());
        let outflow = if let Some((coefficient, _, width)) = &recipe.ports.valve {
            if recipe.values[&width.id] <= 0.0 || recipe.values[&coefficient.id] < 0.0 {
                return Err(contract(
                    "directional valve needs nonnegative coefficient and positive pressure width",
                ));
            }
            pse_quantity::admission::require_same_contract(
                width.quantity,
                recipe.ports.pressure.quantity,
                &physical.quantities,
            )
            .map_err(super::math)?;
            sources.valve_laws.push(
                pse_relations::generated::authored::directional_valve_laws::Row {
                    model_id: model,
                    name: valve_name.clone(),
                    transition_width_id: width.id,
                },
            );
            format!(
                "valve_k * sqrt(valve_width) * kernel.{valve_name}((pressure - downstream) / valve_width)"
            )
        } else {
            "outflow".into()
        };
        let formals: Vec<_> = roles
            .iter()
            .map(
                |(r, p)| m::AuthoredComputationModelsFieldDefinitionsItemFormalsItem {
                    path: (*r).into(),
                    quantity_id: p.quantity.as_id(),
                },
            )
            .collect();
        let slots: Vec<_> = roles
            .iter()
            .map(|(_, p)| {
                Ok(
                    m::AuthoredComputationModelsFieldCasesItemInstancesItemSlotsItem {
                        source_id: p.id,
                        formal_quantity_id: p.quantity.as_id(),
                        formal_unit_id: physical
                            .quantities
                            .quantity_type(p.quantity)
                            .map_err(super::math)?
                            .canonical_unit
                            .as_id(),
                    },
                )
            })
            .collect::<Result<_, WorkflowError>>()?;
        let mut definitions = vec![];
        let mut instances = vec![];
        let mut rows = vec![];
        let mut add = |role: &str,
                       expression: String,
                       quantity: QuantityTypeId,
                       mut providers: Vec<String>,
                       visible: bool| {
            if expression.contains(&format!("kernel.{valve_name}(")) {
                providers.push(valve_name.clone());
            }
            definitions.push(m::AuthoredComputationModelsFieldDefinitionsItem {
                definition_id: name(&format!("definition.{role}")),
                sources: vec![expression],
                formals: formals.clone(),
                domains: vec![],
                groups: vec![],
                providers,
                units: vec![],
                literals: vec![],
            });
            instances.push(m::AuthoredComputationModelsFieldCasesItemInstancesItem {
                instance_id: name(&format!("instance.{role}")),
                definition_id: name(&format!("definition.{role}")),
                slots: slots.clone(),
                contributions: if visible {
                    vec![
                        m::AuthoredComputationModelsFieldCasesItemInstancesItemContributionsItem {
                            output: 0,
                            row_id: Some(name(&format!("row.{role}"))),
                            scale: 1.0,
                        },
                    ]
                } else {
                    vec![]
                },
            });
            if visible {
                rows.push(m::AuthoredComputationModelsFieldCasesItemRowsItem {
                    row_id: name(&format!("row.{role}")),
                    quantity_id: quantity.as_id(),
                    lower: None,
                    upper: None,
                });
            }
        };
        for (role, expr, qty, providers) in [
            (
                "amount_in",
                "inflow".into(),
                recipe.functions.amount_rate,
                vec![],
            ),
            (
                "amount_out",
                outflow.clone(),
                recipe.functions.amount_rate,
                vec![],
            ),
            (
                "energy_in",
                "inflow * inlet_enthalpy".into(),
                recipe.functions.energy_rate,
                vec![],
            ),
            (
                "energy_out",
                format!("({outflow}) * ({he})"),
                recipe.functions.energy_rate,
                vec![enthalpy.name.clone()],
            ),
            ("heat", "heat".into(), recipe.functions.energy_rate, vec![]),
        ] {
            add(role, expr, qty, providers, false);
        }
        add(
            "closure_n",
            "density * volume - n".into(),
            recipe.functions.amount,
            vec![],
            true,
        );
        add(
            "closure_u",
            format!("n * ({he} - pressure / density) - u"),
            recipe.functions.energy,
            vec![enthalpy.name.clone()],
            true,
        );
        add(
            "closure_p",
            format!("({pe}) - pressure"),
            recipe.functions.pressure,
            vec![pressure.name.clone()],
            true,
        );
        for (role, p) in roles.iter().take(5) {
            add(
                &format!("initial.{role}"),
                format!("{role}0"),
                p.quantity,
                vec![],
                true,
            );
            add(
                &format!("output.{role}"),
                (*role).into(),
                p.quantity,
                vec![],
                true,
            );
        }
        let variables = roles
            .iter()
            .take(5)
            .map(
                |(_, p)| m::AuthoredComputationModelsFieldCasesItemVariablesItem {
                    port: m::AuthoredComputationModelsFieldCasesItemVariablesItemPort {
                        symbol_id: p.id,
                        quantity_id: p.quantity.as_id(),
                        unit_id: p.unit.as_id(),
                    },
                    fixed: false,
                    domain: NativeVariableDomain::Continuous,
                    lower: None,
                    upper: None,
                },
            )
            .collect();
        let parameters = roles
            .iter()
            .skip(5)
            .map(
                |(_, p)| m::AuthoredComputationModelsFieldCasesItemParametersItem {
                    symbol_id: p.id,
                    quantity_id: p.quantity.as_id(),
                    unit_id: p.unit.as_id(),
                },
            )
            .collect();
        let values = recipe
            .values
            .iter()
            .map(
                |(&symbol_id, &value)| m::AuthoredComputationModelsFieldCasesItemValuesItem {
                    symbol_id,
                    value,
                },
            )
            .collect();
        row.definitions.extend(definitions);
        row.cases.push(m::AuthoredComputationModelsFieldCasesItem {
            case_id: name("case"),
            name: "conserved homogeneous vessel".into(),
            variables,
            parameters,
            instances,
            rows,
            objective: None,
            values,
        });
        let states = roles
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, (r, p))| d::AuthoredDynamicCasesFieldStatesItem {
                symbol_id: p.id,
                differential: i < 2,
                initial_row: name(&format!("row.initial.{r}")),
                offset: 0.0,
                scale: recipe.state_scales[i],
                residual_scale: if i < 2 {
                    1.0
                } else {
                    recipe.residual_scales[i - 2]
                },
            })
            .collect();
        sources.dynamics.push(d::Row {
            dynamic_id: recipe.id,
            model_id: model,
            case_id: name("case"),
            time_id: recipe.ports.time.id,
            time_origin: None,
            states,
            parameters: vec![
                recipe.ports.inflow.id,
                recipe.ports.outflow.id,
                recipe.ports.inlet_enthalpy.id,
                recipe.ports.heat.id,
            ],
            outputs: state_names
                .iter()
                .map(|r| name(&format!("row.output.{r}")))
                .collect(),
            modes: vec![d::AuthoredDynamicCasesFieldModesItem {
                rhs_rows: ["rate_n", "rate_u", "closure_n", "closure_u", "closure_p"]
                    .iter()
                    .map(|r| name(&format!("row.{r}")))
                    .collect(),
                events: vec![],
            }],
        });
        for (i, rate, state, qty, terms) in [
            (
                0,
                "rate_n",
                recipe.ports.n.id,
                recipe.functions.amount_rate,
                vec![
                    ("amount_in", BalanceRole::Inlet),
                    ("amount_out", BalanceRole::Outlet),
                ],
            ),
            (
                1,
                "rate_u",
                recipe.ports.u.id,
                recipe.functions.energy_rate,
                vec![
                    ("energy_in", BalanceRole::Inlet),
                    ("energy_out", BalanceRole::Outlet),
                    ("heat", BalanceRole::HeatIn),
                ],
            ),
        ] {
            sources.balances.push(b::Row{balance_id:name(&format!("row.{rate}")),model_id:model,case_id:name("case"),quantity_id:qty.as_id(),accumulation:Some(state),tolerance:recipe.balance_tolerances[i],integral_tolerance:Some(recipe.balance_tolerances[i]),provenance:"declared homogeneous vessel amount/internal-energy contributions; empirical validity unestablished".into(),terms:terms.into_iter().map(|(r,role)|b::AuthoredPhysicalBalancesFieldTermsItem{ multiplier:1.0,source_id:name(&format!("physical.{r}")),role,mode:None,transfer_id:None,instance_id:name(&format!("instance.{r}")),output:0}).collect(),impulses:vec![]});
        }
        sources.providers.extend([pressure, enthalpy]);
        Ok((row, sources))
    }
}

fn composition_coordinates(provider: &NativeProviderDeclaration) -> Result<String, WorkflowError> {
    let dependent = provider
        .components
        .iter()
        .find(|c| c.species_id == provider.dependent_species);
    if provider.components.len() != 3 || dependent.is_none_or(|c| c.pcsaft_cas != "74-98-6") {
        return Err(contract(
            "the vessel recipe requires methane/ethane coordinates and dependent propane",
        ));
    }
    let names: Vec<_> = provider
        .components
        .iter()
        .filter(|c| c.species_id != provider.dependent_species)
        .map(|c| match c.pcsaft_cas.as_str() {
            "74-82-8" => Ok("methane"),
            "74-84-0" => Ok("ethane"),
            _ => Err(contract("unsupported species in the vessel recipe")),
        })
        .collect::<Result<_, _>>()?;
    if names.len() != 2 || names[0] == names[1] {
        return Err(contract("duplicate vessel composition species"));
    }
    Ok(names.join(","))
}

impl ModelBuilder {
    /// Apply the vessel declaration producer to the ordinary editable model draft.
    /// # Errors
    /// Invalid recipe or conflicting source identities.
    pub fn vessel(&mut self, recipe: VesselRecipe) -> Result<&mut Self, WorkflowError> {
        let (row, sources) = recipe.declarations(self.row.model_id, self.physical_context())?;
        let mut merged = self.sources.clone();
        merged.merge(&sources)?;
        self.row.definitions.extend(row.definitions);
        self.row.cases.extend(row.cases);
        self.sources = merged;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vessel_declarations_map_actual_species_coordinate_order() {
        let providers: Vec<NativeProviderDeclaration> = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/plan14/providers.json"
        ))
        .unwrap();
        let mut provider = providers[0].clone();
        assert_eq!(
            composition_coordinates(&provider).unwrap(),
            "methane,ethane"
        );
        provider.components.reverse();
        assert_eq!(
            composition_coordinates(&provider).unwrap(),
            "ethane,methane"
        );
        provider.dependent_species = provider.components[1].species_id;
        assert!(composition_coordinates(&provider).is_err());
    }
}
