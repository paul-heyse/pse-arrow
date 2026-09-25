// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Additional generated workflow declarations; no parallel measurement or model DTOs.
use super::{ModelBuilder, ProviderBinding, WorkflowError, contract, relation};
use pse_ids::SemanticId;
use pse_model::HeapUsage;
use pse_relations::{columnar::FieldCheckedBatch, generated::authored::*};
use std::{collections::BTreeMap, sync::Arc};
/// Exact dynamic declaration from the schema registry.
pub type DynamicDeclaration = dynamic_cases::Row;
/// Exact fitting declaration from the schema registry.
pub type FitDeclaration = fit_cases::Row;
/// A native library factory selector, never a Python callback.
pub type NativeProviderDeclaration = native_providers::Row;
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
/// Generated source relations accompanying an authored computation model.
pub struct Sources {
    /// Explicit nonreversing valve closure and its authored pressure width.
    #[serde(default)]
    pub valve_laws: Vec<directional_valve_laws::Row>,
    /// Registry-owned numerical magnitudes and budgets selected with the model/case.
    #[serde(default)]
    pub numerics: Vec<numerical_requirements::Row>,
    /// Explicit provider-output binding for selected property scaling defaults.
    #[serde(default)]
    pub scaling_bindings: Vec<provider_scaling_bindings::Row>,
    /// Authored property defaults; interpreted only through an explicit binding.
    #[serde(default)]
    pub scaling_defaults: Vec<default_scaling::Row>,
    /// Reusable templates and the explicitly selected instance root.
    #[serde(default)]
    pub composition: super::composition::CompositionDeclarations,
    /// Signed physical contributions, distinct from numerical equation residuals.
    #[serde(default)]
    pub balances: Vec<super::BalanceDeclaration>,
    /// Dynamic cases over declared model outputs.
    #[serde(default)]
    pub dynamics: Vec<DynamicDeclaration>,
    /// Observation and shared-parameter bindings.
    #[serde(default)]
    pub fits: Vec<FitDeclaration>,
    /// Native provider factories.
    #[serde(default)]
    pub providers: Vec<NativeProviderDeclaration>,
    /// Measurements from authored datasets.
    #[serde(default)]
    pub observations: Vec<observations::Row>,
    /// Dataset provenance.
    #[serde(default)]
    pub datasets: Vec<datasets::Row>,
}
impl Sources {
    pub(crate) fn merge(&mut self, other: &Self) -> Result<(), WorkflowError> {
        self.composition.merge(&other.composition)?;
        macro_rules! merge {
            ($field:ident,$key:expr) => {
                for row in &other.$field {
                    if let Some(old) = self.$field.iter().find(|old| $key(old) == $key(row)) {
                        if old != row {
                            return Err(contract(
                                "conflicting authored source identities in one run",
                            ));
                        }
                    } else {
                        self.$field.push(row.clone());
                    }
                }
            };
        }
        merge!(scaling_bindings, |r: &provider_scaling_bindings::Row| r
            .binding_id);
        merge!(scaling_defaults, |r: &default_scaling::Row| (
            r.property_package_id,
            r.property_kind_id,
            r.index.clone()
        ));
        merge!(valve_laws, |r: &directional_valve_laws::Row| (
            r.model_id,
            r.name.clone()
        ));
        merge!(numerics, |r: &numerical_requirements::Row| r.requirement_id);
        merge!(balances, |r: &super::BalanceDeclaration| r.balance_id);
        merge!(dynamics, |r: &DynamicDeclaration| r.dynamic_id);
        merge!(fits, |r: &FitDeclaration| r.fit_id);
        merge!(providers, |r: &NativeProviderDeclaration| (
            r.model_id,
            r.name.clone()
        ));
        merge!(observations, |r: &observations::Row| r.observation_id);
        merge!(datasets, |r: &datasets::Row| r.dataset_id);
        Ok(())
    }

    pub(crate) fn canonicalize(&mut self, model: SemanticId) -> Result<(), WorkflowError> {
        self.composition.canonicalize()?;
        self.scaling_bindings.sort_by_key(|r| r.binding_id);
        self.scaling_defaults
            .sort_by_key(|r| (r.property_package_id, r.property_kind_id, r.index.clone()));
        if self.scaling_bindings.iter().any(|r| r.model_id != model)
            || self
                .scaling_bindings
                .windows(2)
                .any(|w| w[0].binding_id == w[1].binding_id)
            || self.scaling_defaults.windows(2).any(|w| {
                (w[0].property_package_id, w[0].property_kind_id, &w[0].index)
                    == (w[1].property_package_id, w[1].property_kind_id, &w[1].index)
            })
        {
            return Err(contract("duplicate or foreign numerical default binding"));
        }
        self.numerics.sort_by_key(|r| r.requirement_id);
        self.balances.sort_by_key(|b| b.balance_id);
        self.dynamics.sort_by_key(|x| x.dynamic_id);
        self.fits.sort_by_key(|x| x.fit_id);
        self.valve_laws.sort_by(|a, b| a.name.cmp(&b.name));
        self.providers.sort_by(|a, b| a.name.cmp(&b.name));
        self.observations.sort_by_key(|x| x.observation_id);
        self.datasets.sort_by_key(|x| x.dataset_id);
        if self.valve_laws.iter().any(|r| r.model_id != model)
            || self.valve_laws.windows(2).any(|w| w[0].name == w[1].name)
            || self
                .valve_laws
                .iter()
                .any(|v| self.providers.iter().any(|p| p.name == v.name))
            || self.numerics.iter().any(|r| r.model_id != model)
            || self
                .numerics
                .windows(2)
                .any(|r| r[0].requirement_id == r[1].requirement_id)
            || self.balances.iter().any(|b| b.model_id != model)
            || self
                .balances
                .windows(2)
                .any(|w| w[0].balance_id == w[1].balance_id)
            || self.dynamics.iter().any(|r| r.model_id != model)
            || self.fits.iter().any(|r| r.model_id != model)
            || self.providers.iter().any(|r| r.model_id != model)
            || self
                .dynamics
                .windows(2)
                .any(|w| w[0].dynamic_id == w[1].dynamic_id)
            || self.fits.windows(2).any(|w| w[0].fit_id == w[1].fit_id)
            || self.providers.windows(2).any(|w| w[0].name == w[1].name)
            || self
                .observations
                .windows(2)
                .any(|w| w[0].observation_id == w[1].observation_id)
            || self
                .datasets
                .windows(2)
                .any(|w| w[0].dataset_id == w[1].dataset_id)
            || self
                .observations
                .iter()
                .any(|o| !self.datasets.iter().any(|d| d.dataset_id == o.dataset_id))
        {
            return Err(contract(
                "workflow declaration ownership, duplicates or missing dataset",
            ));
        }
        Ok(())
    }
    pub(crate) fn bytes(&self) -> usize {
        self.dynamics
            .owned_bytes()
            .saturating_add(self.composition.bytes())
            .saturating_add(self.balances.owned_bytes())
            .saturating_add(self.numerics.owned_bytes())
            .saturating_add(self.scaling_bindings.owned_bytes())
            .saturating_add(self.scaling_defaults.owned_bytes())
            .saturating_add(self.fits.owned_bytes())
            .saturating_add(self.providers.owned_bytes())
            .saturating_add(self.valve_laws.owned_bytes())
            .saturating_add(self.observations.owned_bytes())
            .saturating_add(self.datasets.owned_bytes())
    }
    pub(crate) fn tables(
        &self,
        registry: &pse_schema::Registry,
    ) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
        let mut out = self.composition.tables(registry)?;
        macro_rules! rows {
            ($module:ident,$field:ident) => {
                if !self.$field.is_empty() {
                    let mut b = $module::Builder::with_registry(registry, self.$field.len())
                        .map_err(relation)?;
                    for row in &self.$field {
                        b.push(row.clone()).map_err(relation)?;
                    }
                    out.insert($module::RELATION_ID, b.finish().map_err(relation)?);
                }
            };
        }
        rows!(physical_balances, balances);
        rows!(numerical_requirements, numerics);
        rows!(provider_scaling_bindings, scaling_bindings);
        rows!(default_scaling, scaling_defaults);
        rows!(dynamic_cases, dynamics);
        rows!(fit_cases, fits);
        rows!(native_providers, providers);
        rows!(directional_valve_laws, valve_laws);
        rows!(observations, observations);
        rows!(datasets, datasets);
        Ok(out)
    }
}
impl ModelBuilder {
    /// Declare a nonreversing C2 valve shape with an explicit positive pressure width.
    pub fn directional_valve_law(&mut self, row: directional_valve_laws::Row) -> &mut Self {
        self.sources.valve_laws.push(row);
        self
    }
    /// Add a registry-owned numerical requirement.
    pub fn numerical_requirement(&mut self, row: numerical_requirements::Row) -> &mut Self {
        self.sources.numerics.push(row);
        self
    }
    /// Bind an exact provider output to a property scaling default.
    pub fn provider_scaling(&mut self, row: provider_scaling_bindings::Row) -> &mut Self {
        self.sources.scaling_bindings.push(row);
        self
    }
    /// Retain a property default; it has no effect until explicitly bound.
    pub fn default_scaling(&mut self, row: default_scaling::Row) -> &mut Self {
        self.sources.scaling_defaults.push(row);
        self
    }

    /// Add one authoritative physical contribution balance.
    pub fn balance(&mut self, row: super::BalanceDeclaration) -> &mut Self {
        self.sources.balances.push(row);
        self
    }

    /// Add a registry-generated dynamics declaration over one selected case.
    pub fn dynamics(&mut self, row: DynamicDeclaration) -> &mut Self {
        self.sources.dynamics.push(row);
        self
    }
    /// Add a registry-generated fitting declaration using authored observations.
    pub fn fit(&mut self, row: FitDeclaration) -> &mut Self {
        self.sources.fits.push(row);
        self
    }
    /// Add a registry-generated measurement, preserving identity and provenance.
    pub fn observation(&mut self, row: observations::Row) -> &mut Self {
        self.sources.observations.push(row);
        self
    }
    /// Add the exact dataset provenance declaration.
    pub fn dataset(&mut self, row: datasets::Row) -> &mut Self {
        self.sources.datasets.push(row);
        self
    }
    /// Select a concrete library factory through a durable typed declaration.
    pub fn native_provider(&mut self, row: NativeProviderDeclaration) -> &mut Self {
        self.sources.providers.push(row);
        self
    }
}
pub(super) fn factory(
    row: &NativeProviderDeclaration,
    q: &pse_quantity::QuantityRegistry,
    materials: &super::CompositionDeclarations,
) -> Result<ProviderBinding, WorkflowError> {
    if row.kind != "feos-pcsaft-dippr" {
        return Err(super::composition::unsupported(
            [row.model_id],
            "unknown native provider factory",
        ));
    }
    if row.formulation
        != pse_relations::generated::enums::ThermodynamicFormulation::HomogeneousDensity
    {
        return Err(super::composition::unsupported(
            [row.model_id],
            "selected phase-equilibrium formulation",
        ));
    }
    let port = |id, quantity, unit| pse_kernels::Port {
        id,
        quantity: pse_quantity::QuantityTypeId::from_id(quantity),
        unit: pse_quantity::UnitId::from_id(unit),
    };
    let inputs: Vec<_> = row
        .inputs
        .iter()
        .map(|p| port(p.symbol_id, p.quantity_id, p.unit_id))
        .collect();
    let outputs: Vec<_> = row
        .outputs
        .iter()
        .map(|p| port(p.symbol_id, p.quantity_id, p.unit_id))
        .collect();
    let range = |values: &[f64]| {
        let [lower, upper] = values else {
            return Err(contract("envelope interval requires two endpoints"));
        };
        pse_kernels::envelope::Interval::new(*lower, *upper).map_err(|e| contract(e.to_string()))
    };
    let envelope = pse_kernels::envelope::StateEnvelope {
        temperature: range(&row.envelope.temperature)?,
        density: range(&row.envelope.density)?,
        pressure: range(&row.envelope.pressure)?,
        composition: row
            .envelope
            .composition
            .iter()
            .map(|v| range(v))
            .collect::<Result<Vec<_>, _>>()?,
        provenance: row.envelope.provenance.clone(),
    };
    use pse_kernels::feos::{ComponentBinding, FeosData, FeosKinds, FeosPorts};
    let kind = pse_quantity::QuantityKindId::from_id;
    let ports = FeosPorts {
        envelope,
        inputs,
        outputs,
        components: row
            .components
            .iter()
            .map(|c| ComponentBinding {
                species: c.species_id,
                pcsaft_cas: c.pcsaft_cas.clone(),
                ideal_gas_cas: c.ideal_gas_cas.clone(),
            })
            .collect(),
        dependent_species: row.dependent_species,
        enthalpy_reference: pse_quantity::ReferenceStateId::from_id(row.enthalpy_reference),
        entropy_reference: pse_quantity::ReferenceStateId::from_id(row.entropy_reference),
        kinds: FeosKinds {
            temperature: kind(row.quantity_kinds.temperature),
            density: kind(row.quantity_kinds.density),
            fraction: kind(row.quantity_kinds.fraction),
            pressure: kind(row.quantity_kinds.pressure),
            enthalpy: kind(row.quantity_kinds.enthalpy),
            entropy: kind(row.quantity_kinds.entropy),
            ln_fugacity: kind(row.quantity_kinds.ln_fugacity),
        },
        data: FeosData {
            pcsaft: row.data.pcsaft.clone(),
            ideal_gas: row.data.ideal_gas.clone(),
            binary: row.data.binary.clone(),
            provenance: row.data.provenance.clone(),
            missing_interactions: row.data.missing_interactions,
        },
        formulation: row.formulation,
        stability: row.stability,
    };
    let package = pse_kernels::feos::FeosPackage::new(ports, q).map_err(|e| {
        super::composition::invalid(
            std::iter::once(row.model_id).chain(row.components.iter().map(|c| c.species_id)),
            e.to_string(),
        )
    })?;
    if let Some(system) = row.material_system_id {
        for record in package.records() {
            let species = materials
                .species
                .iter()
                .find(|s| s.species_id == record.species)
                .ok_or_else(|| {
                    super::composition::invalid(
                        [system, record.species],
                        "provider species declaration missing",
                    )
                })?;
            if species.mw.is_some_and(|mw| {
                !mw.is_finite() || (mw - record.molar_mass).abs() > 1e-8 * record.molar_mass
            }) || species
                .formula
                .as_ref()
                .zip(record.formula.as_ref())
                .is_some_and(|(a, b)| a != b)
            {
                return Err(super::composition::invalid(
                    [system, record.species],
                    "authored chemical identity differs from bound parameter record",
                ));
            }
        }
    }
    let registration = pse_kernels::Registration::new(Arc::new(package), q)
        .map_err(|e| contract(e.to_string()))?;
    let output = usize::try_from(row.output).map_err(|_| contract("provider output index"))?;
    if output >= registration.spec().outputs.len() {
        return Err(contract("provider output index"));
    }
    Ok(ProviderBinding {
        registration,
        output,
    })
}

#[cfg(test)]
mod tests {
    use pse_relations::{
        columnar::RelationRow,
        generated::{
            enums::*,
            runtime::{solve_metrics, solve_runs},
        },
    };
    #[test]
    fn shared_execution_tags_and_unavailable_evidence_roundtrip_arrow_and_json() {
        let registry = pse_schema::registry().unwrap();
        let id = pse_ids::SemanticId::from_bytes([1; 16]);
        let mut expected = vec![];
        for (i, termination) in NativeTermination::ALL.into_iter().enumerate() {
            let row = solve_runs::Row {
                run_id: id,
                step: i as i64,
                model_id: Some(id),
                revision: None,
                case_id: Some(id),
                backend: Some(NativeBackend::ALL[i % NativeBackend::ALL.len()]),
                native_code: Some(i as i64),
                native_status: Some("native detail".into()),
                state: NativeRunState::Native,
                termination: Some(termination),
                assurance: NativeAssurance::ALL[i % NativeAssurance::ALL.len()],
                candidate_kind: Some(NativeCandidateKind::ALL[i % NativeCandidateKind::ALL.len()]),
                feasible: None,
                objective: None,
                objective_sense: None,
                objective_quantity_id: None,
                validation_error: None,
                error: None,
                transformation: None,
            };
            let wire = serde_json::to_value(termination).unwrap();
            assert_eq!(
                serde_json::from_value::<NativeTermination>(wire).unwrap(),
                termination
            );
            assert!(
                serde_json::from_value::<NativeTermination>(serde_json::json!(
                    "unrecognized-native-success"
                ))
                .is_err()
            );
            expected.push(row);
        }
        let mut builder = solve_runs::Builder::with_registry(registry, expected.len()).unwrap();
        for row in &expected {
            builder.push(row.clone()).unwrap();
        }
        assert_eq!(
            solve_runs::Row::rows(&builder.finish().unwrap()).unwrap(),
            expected
        );
        let mut builder =
            solve_metrics::Builder::with_registry(registry, EvidenceUnavailableReason::ALL.len())
                .unwrap();
        for (step, reason) in EvidenceUnavailableReason::ALL.into_iter().enumerate() {
            builder
                .push(solve_metrics::Row {
                    run_id: id,
                    step: step as i64,
                    namespace: "native".into(),
                    name: "optional metric".into(),
                    kind: NativeMetricKind::Unavailable,
                    real: None,
                    integer: None,
                    boolean: None,
                    text: None,
                    unavailable: Some(reason),
                })
                .unwrap();
        }
        let rows = solve_metrics::Row::rows(&builder.finish().unwrap()).unwrap();
        assert_eq!(
            rows.iter()
                .map(|r| r.unavailable.unwrap())
                .collect::<Vec<_>>(),
            EvidenceUnavailableReason::ALL
        );
    }
}
