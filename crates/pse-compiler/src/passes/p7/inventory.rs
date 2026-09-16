// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Inputs, invalid};
use crate::{
    CompilerError,
    passes::native_rows::{self, AlgorithmInputs},
};
use datafusion::{
    arrow::array::{Array, StringArray},
    logical_expr::{LogicalPlanBuilder, col},
};
use pse_catalog::session::SnapshotSession;
use pse_catalog::session::scalar;
use pse_ids::{CancellationToken, MemoryReserver};
use pse_relations::{
    columnar::RelationRow,
    generated::{compiled as c, inferred as i, normalized as n, reference as r},
};
use pse_schema::{
    Registry,
    model::{RelationKey, RelationSpec},
};
use std::collections::BTreeMap;

pub(super) fn decode<T: RelationRow>(
    inputs: &Inputs,
    registry: &Registry,
    spec: &RelationSpec,
) -> Result<Vec<T>, CompilerError> {
    let batch = inputs
        .get(&spec.key)
        .ok_or_else(|| invalid(format!("P7 lacks {}", spec.key.qualified_name())))?;
    if T::relation(registry)?.key != spec.key {
        return Err(invalid("P7 typed input declaration differs"));
    }
    Ok(T::rows(batch)?)
}

struct Loader<'a> {
    inputs: &'a Inputs,
    registry: &'a Registry,
    session: &'a SnapshotSession,
    cancel: &'a CancellationToken,
    arguments: AlgorithmInputs,
    keys: BTreeMap<RelationKey, Vec<String>>,
}
impl Loader<'_> {
    async fn capture_keys(&mut self) -> Result<(), CompilerError> {
        for (key, input) in self.inputs {
            if self.keys.contains_key(key) {
                continue;
            }
            let spec = self
                .registry
                .relation(&key.qualified_name())
                .ok_or_else(|| invalid("native source declaration absent"))?;
            let session = self.session.with_checked_role_inputs(
                BTreeMap::from([("realization_source_keys".to_owned(), input.clone())]),
                self.cancel,
            )?;
            let plan = LogicalPlanBuilder::from(session.scan_role("realization_source_keys")?)
                .project([scalar::key(
                    spec.primary_key
                        .iter()
                        .map(|name| (*name, col(*name)))
                        .collect(),
                )
                .alias("source_key")])
                .and_then(LogicalPlanBuilder::build)
                .map_err(native_rows::engine)?;
            let completed = self.arguments.execute(plan, &session, self.cancel).await?;
            let mut keys = Vec::new();
            for batch in completed.batches() {
                let values = batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| invalid("source keys are not Utf8"))?;
                for row in 0..batch.num_rows() {
                    if values.is_null(row) {
                        return Err(invalid("source key is null"));
                    }
                    keys.push(values.value(row).to_owned());
                }
            }
            self.keys.insert(*key, keys);
        }
        Ok(())
    }
    async fn optional<T: RelationRow>(&mut self) -> Result<Vec<T>, CompilerError> {
        if self.inputs.contains_key(&T::relation(self.registry)?.key) {
            self.rows().await
        } else {
            Ok(Vec::new())
        }
    }
    async fn rows<T: RelationRow>(&mut self) -> Result<Vec<T>, CompilerError> {
        let spec = T::relation(self.registry)?;
        let input = self
            .inputs
            .get(&spec.key)
            .ok_or_else(|| invalid(format!("P7 lacks {}", spec.key.qualified_name())))?
            .clone();
        let session = self.session.with_checked_role_inputs(
            BTreeMap::from([("realization_input".to_owned(), input)]),
            self.cancel,
        )?;
        let plan = LogicalPlanBuilder::from(session.scan_role("realization_input")?)
            .build()
            .map_err(native_rows::engine)?;
        let selected = native_rows::keyed_rows::<T>(
            &mut self.arguments,
            plan,
            &session,
            self.registry,
            self.cancel,
        )
        .await?;
        let (rows, keys) = selected
            .into_iter()
            .map(|source| (source.row, source.key))
            .unzip();
        self.keys.insert(spec.key, keys);
        Ok(rows)
    }
}
pub(super) struct Inventory {
    pub parameter_bindings: Vec<c::method_parameter_bindings::Row>,
    pub parameter_values: Vec<n::parameter_values::Row>,
    pub method_realizations: Vec<c::method_realizations::Row>,
    pub method_resolutions: Vec<i::method_resolutions::Row>,
    pub method_specs: Vec<r::method_specs::Row>,
    pub method_provisions: Vec<r::method_provisions::Row>,
    pub property_requirements: Vec<i::property_requirements::Row>,
    pub arguments: AlgorithmInputs,
    pub source_keys: BTreeMap<RelationKey, Vec<String>>,
    pub instances: Vec<i::instances::Row>,
    pub prospective: Vec<n::instance_bindings::Row>,
    pub templates: Vec<n::templates::Row>,
    pub symbols: Vec<n::template_symbols::Row>,
    pub contracts: Vec<n::template_symbol_contracts::Row>,
    pub bodies: Vec<n::template_symbol_expressions::Row>,
    pub equations: Vec<n::template_equations::Row>,
    pub instance_equations: Vec<n::instance_equations::Row>,
    pub submodels: Vec<n::template_submodels::Row>,
    pub parameters: Vec<n::template_params::Row>,
    pub features: Vec<n::template_features::Row>,
    pub ports: Vec<n::template_ports::Row>,
    pub actual_ports: Vec<i::ports::Row>,
    pub connections: Vec<n::connections::Row>,
    pub connection_bindings: Vec<r::connection_bindings::Row>,
    pub port_members: Vec<i::port_members::Row>,
    pub port_state_targets: Vec<i::port_state_targets::Row>,
    pub port_state_domains: Vec<i::port_state_domains::Row>,
    pub port_member_domains: Vec<i::port_member_domains::Row>,
    pub domain_bindings: Vec<n::instance_domain_bindings::Row>,
    pub domains: Vec<n::domains::Row>,
    pub members: Vec<n::domain_members::Row>,
    pub products: Vec<n::domain_products::Row>,
    pub tuples: Vec<i::valid_index_tuples::Row>,
    pub configuration: Vec<n::config_values::Row>,
    pub feature_values: Vec<i::instance_features::Row>,
    pub sources: Vec<n::expression_sources::Row>,
    pub indices: Vec<n::expression_index_bindings::Row>,
    pub equation_nodes: Vec<n::equation_nodes::Row>,
    pub predicates: Vec<i::predicate_outcomes::Row>,
    pub predicate_axes: Vec<i::predicate_axes::Row>,
    pub paths: Vec<n::expression_paths::Row>,
    pub targets: Vec<i::path_targets::Row>,
    pub guards: Vec<n::template_guards::Row>,
    pub derivatives: Vec<n::template_derivatives::Row>,
    pub contributions: Vec<n::template_contributions::Row>,
    pub contribution_contracts: Vec<n::template_contribution_contracts::Row>,
    pub scopes: Vec<n::template_scopes::Row>,
    pub scope_bindings: Vec<i::scope_bindings::Row>,
}
impl Inventory {
    pub(super) async fn load(
        inputs: &Inputs,
        registry: &Registry,
        session: &SnapshotSession,
        reserver: &dyn MemoryReserver,
        cancel: &CancellationToken,
    ) -> Result<Self, CompilerError> {
        let mut loader = Loader {
            inputs,
            registry,
            session,
            cancel,
            arguments: AlgorithmInputs::new(reserver, "P7:typed-arguments"),
            keys: BTreeMap::new(),
        };
        let result = Self {
            parameter_bindings: loader.optional().await?,
            parameter_values: loader.optional().await?,
            method_realizations: loader.optional().await?,
            method_resolutions: loader.optional().await?,
            method_specs: loader.optional().await?,
            method_provisions: loader.optional().await?,
            property_requirements: loader.optional().await?,
            instances: loader.rows::<i::instances::Row>().await?,
            prospective: loader.rows::<n::instance_bindings::Row>().await?,
            templates: loader.rows::<n::templates::Row>().await?,
            symbols: loader.rows::<n::template_symbols::Row>().await?,
            contracts: loader.rows::<n::template_symbol_contracts::Row>().await?,
            bodies: loader.rows::<n::template_symbol_expressions::Row>().await?,
            equations: loader.rows::<n::template_equations::Row>().await?,
            instance_equations: loader.rows::<n::instance_equations::Row>().await?,
            submodels: loader.rows::<n::template_submodels::Row>().await?,
            parameters: loader.rows::<n::template_params::Row>().await?,
            features: loader.rows::<n::template_features::Row>().await?,
            ports: loader.rows::<n::template_ports::Row>().await?,
            actual_ports: loader.rows::<i::ports::Row>().await?,
            port_members: loader.rows::<i::port_members::Row>().await?,
            connections: loader.rows::<n::connections::Row>().await?,
            connection_bindings: loader.rows::<r::connection_bindings::Row>().await?,
            port_state_targets: loader.rows::<i::port_state_targets::Row>().await?,
            port_state_domains: loader.rows::<i::port_state_domains::Row>().await?,
            port_member_domains: loader.rows::<i::port_member_domains::Row>().await?,
            domain_bindings: loader.rows::<n::instance_domain_bindings::Row>().await?,
            domains: loader.rows::<n::domains::Row>().await?,
            members: loader.rows::<n::domain_members::Row>().await?,
            products: loader.rows::<n::domain_products::Row>().await?,
            tuples: loader.rows::<i::valid_index_tuples::Row>().await?,
            configuration: loader.rows::<n::config_values::Row>().await?,
            feature_values: loader.rows::<i::instance_features::Row>().await?,
            sources: loader.rows::<n::expression_sources::Row>().await?,
            indices: loader.rows::<n::expression_index_bindings::Row>().await?,
            equation_nodes: loader.rows::<n::equation_nodes::Row>().await?,
            predicates: loader.rows::<i::predicate_outcomes::Row>().await?,
            predicate_axes: loader.rows::<i::predicate_axes::Row>().await?,
            paths: loader.rows::<n::expression_paths::Row>().await?,
            targets: loader.rows::<i::path_targets::Row>().await?,
            guards: loader.rows::<n::template_guards::Row>().await?,
            derivatives: loader.rows::<n::template_derivatives::Row>().await?,
            contributions: loader.rows::<n::template_contributions::Row>().await?,
            contribution_contracts: loader
                .rows::<n::template_contribution_contracts::Row>()
                .await?,
            scopes: loader.rows::<n::template_scopes::Row>().await?,
            scope_bindings: loader.rows::<i::scope_bindings::Row>().await?,
            source_keys: {
                loader.capture_keys().await?;
                std::mem::take(&mut loader.keys)
            },
            arguments: loader.arguments,
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passes::native_test;
    use pse_ids::FixedBudget;
    use pse_relations::columnar::FieldCheckedBatch;
    use std::sync::Arc;

    #[tokio::test]
    async fn realization_reads_only_its_declared_input_inventory() {
        let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
        let pass = registry.pass("P7@1").unwrap();
        let inputs = pass
            .inputs
            .iter()
            .map(|port| {
                let spec = registry.relation(&port.relation).unwrap();
                (
                    spec.key,
                    FieldCheckedBatch::concat(&registry, spec, &[]).unwrap(),
                )
            })
            .collect::<Inputs>();
        let budget = FixedBudget::new(128 << 20);
        let cancel = CancellationToken::new();
        let (session, _) =
            native_test::session(&registry, inputs.clone(), &budget, &cancel).unwrap();
        Inventory::load(&inputs, &registry, &session, budget.as_ref(), &cancel)
            .await
            .unwrap_or_else(|error| {
                panic!("declared P7 inputs do not satisfy its reader: {error}")
            });
    }
}
