// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated physical trajectory transport, including interrupted native outcomes.
use super::*;

impl ModelingTrajectory {
    /// Materialize a complete checked map once on success, sharing storage across clones.
    pub fn tables(
        &self,
    ) -> Result<Arc<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>>, WorkflowError>
    {
        let mut cached = self
            .inner
            .tables
            .lock()
            .map_err(|_| contract("trajectory transport lock poisoned"))?;
        if let Some(tables) = cached.as_ref() {
            return Ok(tables.clone());
        }
        // Reserve map ownership before allocation. Column buffers retain their own leases.
        let reservation = self
            .inner
            .prepared
            .runtime
            .shared
            .math()
            .reserve("modeling:trajectory-map", source_map_extent(10)?)?;
        let mut encoded = self.encode_tables()?;
        for batch in encoded.values_mut() {
            *batch = batch.clone().with_export_owner(reservation.clone());
        }
        let tables = Arc::new(encoded);
        *cached = Some(tables.clone());
        Ok(tables)
    }
    /// Resolve and share one relation from the successful production materialization.
    pub fn table(
        &self,
        name: &str,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let id = self
            .inner
            .prepared
            .runtime
            .registry
            .relation(name)
            .ok_or_else(|| contract(format!("unknown trajectory table {name}")))?
            .id;
        self.tables()?
            .get(&id)
            .cloned()
            .ok_or_else(|| contract(format!("trajectory table absent: {name}")))
    }
    fn encode_tables(
        &self,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        use pse_model::generated::runtime::{
            computation_runs, response_sensitivities, simulation_events, simulation_samples,
        };
        let p = &self.inner.prepared;
        let product = p.model().compiled();
        let pool = p.runtime.shared.pool();
        let cancel = pse_columnar::CancellationToken::new();
        let _scratch = p.runtime.shared.math().reserve(
            "modeling:trajectory-row-copy",
            p.contract
                .parameters
                .len()
                .checked_mul(size_of::<&pse_math::binding::Target>())
                .and_then(|n| n.checked_add(4096))
                .and_then(|n| {
                    n.checked_add(
                        self.inner
                            .reports
                            .iter()
                            .map(pse_model::HeapUsage::owned_bytes)
                            .max()
                            .unwrap_or(0),
                    )
                })
                .ok_or_else(|| contract("trajectory row scratch extent"))?,
        )?;
        let validation = p.runtime.validation_context()?;
        let mut columns = pse_relations::columnar::Collection::new(
            &p.runtime.registry,
            &pool,
            &cancel,
            &validation,
        );
        columns
            .ensure::<computation_runs::Row>()
            .map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::modeling_checks::Row>()
            .map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::modeling_findings::Row>()
            .map_err(relation)?;
        if let Some(failure) = self.diagnostic() {
            columns
                .push(analysis_tables::finding_row(self.inner.run_id, 0, &failure))
                .map_err(relation)?;
        }
        for check in &self.inner.checks {
            columns.push(check.clone()).map_err(relation)?;
        }
        columns
            .ensure::<pse_model::generated::runtime::modeling_reports::Row>()
            .map_err(relation)?;
        for row in &self.inner.reports {
            columns.push(row.clone()).map_err(relation)?;
        }
        columns
            .ensure::<simulation_samples::Row>()
            .map_err(relation)?;
        columns
            .ensure::<simulation_events::Row>()
            .map_err(relation)?;
        columns
            .ensure::<response_sensitivities::Row>()
            .map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::modeling_trajectory_modes::Row>()
            .map_err(relation)?;
        for (sample, point) in self.inner.report.samples.iter().enumerate() {
            let mode = p
                .modes
                .get(point.mode)
                .ok_or_else(|| contract("trajectory mode absent"))?;
            columns
                .push(
                    pse_model::generated::runtime::modeling_trajectory_modes::Row {
                        run_id: self.inner.run_id,
                        sample: sample as i64,
                        time: point.time,
                        mode: mode.name.clone(),
                    },
                )
                .map_err(relation)?;
        }
        columns
            .push(self.inner.assessment.clone())
            .map_err(relation)?;
        let r = &self.inner.report;
        columns
            .ensure::<pse_model::generated::runtime::trajectory_endpoints::Row>()
            .map_err(relation)?;
        if let Some(end) = &r.endpoint {
            let coverage = &self.inner.coverage;
            columns
                .push(pse_model::generated::runtime::trajectory_endpoints::Row {
                    run_id: self.inner.run_id,
                    requirement: p.profile.endpoint.kind,
                    required_event: p.profile.endpoint.event,
                    event_id: end.event,
                    time: end.point.time,
                    mode: end.point.mode as i64,
                    state_ids: p.coordinates.state.iter().map(|c| c.id).collect(),
                    input_ids: p.coordinates.parameters.iter().map(|c| c.id).collect(),
                    output_ids: p.contract.outputs.clone(),
                    state: p
                        .coordinates
                        .state
                        .iter()
                        .zip(&end.point.state)
                        .map(|(c, v)| v * c.scale + c.offset)
                        .collect(),
                    inputs: p
                        .coordinates
                        .parameters
                        .iter()
                        .zip(&end.inputs)
                        .map(|(c, v)| v * c.scale + c.offset)
                        .collect(),
                    outputs: end.point.outputs.clone(),
                    integrals: end.point.integrals.clone(),
                    input_columns: end.input_columns.iter().map(|c| *c as i64).collect(),
                    endpoint_satisfied: coverage.satisfied,
                    prefix_complete: coverage.prefix_complete,
                    missing_observations: coverage.missing_observations.clone(),
                })
                .map_err(relation)?;
        }
        columns.push(self.inner.header.clone()).map_err(relation)?;
        let parameters = p
            .contract
            .parameters
            .iter()
            .map(|id| {
                product
                    .admitted
                    .case()
                    .parameters()
                    .iter()
                    .chain(product.admitted.case().variables().iter().map(|v| &v.port))
                    .find(|v| v.id == *id)
                    .ok_or_else(|| contract("trajectory parameter port absent"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (sample, point) in r.samples.iter().enumerate() {
            if point.outputs.len() != p.contract.outputs.len()
                || !point.output_sensitivities.is_empty()
                    && point.output_sensitivities.len()
                        != p.contract.outputs.len() * parameters.len()
            {
                return Err(contract("trajectory output or sensitivity extent"));
            }
            for (i, id) in p.contract.outputs.iter().enumerate() {
                let output = product
                    .admitted
                    .outputs
                    .iter()
                    .find(|o| o.row_id() == *id)
                    .ok_or_else(|| contract("trajectory output lineage absent"))?;
                let ModelingOutput::Member(symbol) = output else {
                    return Err(contract("trajectory output is not an authored member"));
                };
                let row = product
                    .admitted
                    .case()
                    .rows()
                    .iter()
                    .find(|r| r.id == *id)
                    .ok_or_else(|| contract("trajectory physical row absent"))?;
                let unit = p
                    .quantities
                    .quantity_type(row.quantity)
                    .map_err(super::super::super::math)?
                    .canonical_unit;
                columns
                    .push(simulation_samples::Row {
                        run_id: self.inner.run_id,
                        sample: sample as i64,
                        time: point.time,
                        symbol_id: *symbol,
                        quantity_id: row.quantity.as_id(),
                        unit_id: unit.as_id(),
                        value: point.outputs[i],
                    })
                    .map_err(relation)?;
                if !point.output_sensitivities.is_empty() {
                    for (j, parameter) in parameters.iter().enumerate() {
                        columns
                            .push(response_sensitivities::Row {
                                run_id: self.inner.run_id,
                                experiment_id: p.solved.instance(),
                                sample: sample as i64,
                                time: Some(point.time),
                                output_id: *symbol,
                                parameter_id: parameter.id,
                                output_unit_id: unit.as_id(),
                                parameter_unit_id: parameter.unit.as_id(),
                                value: point.output_sensitivities[i * parameters.len() + j],
                            })
                            .map_err(relation)?;
                    }
                }
            }
        }
        for (ordinal, event) in r.events.iter().enumerate() {
            if event.before.len() != p.coordinates.state.len()
                || event
                    .after
                    .as_ref()
                    .is_some_and(|v| v.len() != p.coordinates.state.len())
            {
                return Err(contract("trajectory event extent"));
            }
            for (i, state) in p.coordinates.state.iter().enumerate() {
                columns
                    .push(simulation_events::Row {
                        run_id: self.inner.run_id,
                        ordinal: ordinal as i64,
                        event_id: event.event,
                        time: event.time,
                        symbol_id: state.id,
                        before: event.before[i] * state.scale + state.offset,
                        after: event
                            .after
                            .as_ref()
                            .map(|v| v[i] * state.scale + state.offset),
                    })
                    .map_err(relation)?;
            }
        }
        Ok(columns
            .finish()
            .map_err(relation)?
            .into_values()
            .map(|b| (b.relation_id(), b))
            .collect())
    }
}
