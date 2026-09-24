// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Additional generated workflow declarations; no parallel measurement or model DTOs.
use super::{ModelBuilder, ProviderBinding, WorkflowError, contract, relation};
use pse_ids::SemanticId;
use pse_model::{HeapUsage, SemanticFrame};
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
        self.balances.sort_by_key(|b| b.balance_id);
        self.dynamics.sort_by_key(|x| x.dynamic_id);
        self.fits.sort_by_key(|x| x.fit_id);
        self.providers.sort_by(|a, b| a.name.cmp(&b.name));
        self.observations.sort_by_key(|x| x.observation_id);
        self.datasets.sort_by_key(|x| x.dataset_id);
        if self.balances.iter().any(|b| b.model_id != model)
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
    pub(crate) fn frame(&self, h: &mut pse_ids::FramedHasher) {
        self.balances.frame(h);
        self.dynamics.frame(h);
        self.fits.frame(h);
        self.providers.frame(h);
        self.observations.frame(h);
        self.datasets.frame(h);
    }
    pub(crate) fn bytes(&self) -> usize {
        self.dynamics
            .owned_bytes()
            .saturating_add(self.balances.owned_bytes())
            .saturating_add(self.fits.owned_bytes())
            .saturating_add(self.providers.owned_bytes())
            .saturating_add(self.observations.owned_bytes())
            .saturating_add(self.datasets.owned_bytes())
    }
    pub(crate) fn tables(
        &self,
        registry: &pse_schema::Registry,
    ) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
        let mut out = BTreeMap::new();
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
        rows!(dynamic_cases, dynamics);
        rows!(fit_cases, fits);
        rows!(native_providers, providers);
        rows!(observations, observations);
        rows!(datasets, datasets);
        Ok(out)
    }
}
impl ModelBuilder {
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
) -> Result<ProviderBinding, WorkflowError> {
    if row.kind != "feos-light-hydrocarbons" {
        return Err(contract("unknown native provider factory"));
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
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| contract("three component envelope intervals required"))?,
        provenance: row.envelope.provenance.clone(),
    };
    let ports = pse_kernels::feos::FeosPorts {
        envelope,
        inputs: inputs
            .try_into()
            .map_err(|_| contract("FeOS needs four input ports"))?,
        outputs: outputs
            .try_into()
            .map_err(|_| contract("FeOS needs six output ports"))?,
        components: row
            .components
            .clone()
            .try_into()
            .map_err(|_| contract("FeOS component order is methane/ethane/propane"))?,
        caloric_reference: pse_quantity::ReferenceStateId::from_id(row.caloric_reference),
    };
    let package =
        pse_kernels::feos::FeosPackage::new(ports, q).map_err(|e| contract(e.to_string()))?;
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
