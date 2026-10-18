// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected generated trajectory projection, preserving exact physical coordinates.
use super::*;
use crate::workflow::math;

/// Each selected physical integration axis retains its contract parameter's physical
/// identity, including every distinct interval of a scheduled input.
fn sensitivity_parameters(
    profile: &native::Profile,
    parameters: usize,
    axes: &[usize],
) -> Result<Vec<usize>, WorkflowError> {
    let first = profile.columns_at(parameters, profile.start);
    axes.iter()
        .enumerate()
        .map(|(column, axis)| {
            if axes[..column].contains(axis) {
                return Err(contract(
                    "trajectory sensitivity axes repeat a physical column",
                ));
            }
            first
                .iter()
                .enumerate()
                .find_map(|(parameter, start)| {
                    let intervals = profile
                        .schedule
                        .iter()
                        .find(|s| s.parameter == parameter)
                        .map_or(1, |s| s.times.len() + 1);
                    axis.checked_sub(*start)
                        .filter(|offset| *offset < intervals)
                        .map(|_| parameter)
                })
                .ok_or_else(|| contract("trajectory physical sensitivity axis absent"))
        })
        .collect()
}

impl ModelingTrajectory {
    pub(in crate::workflow) fn relation_ids() -> Vec<SemanticId> {
        use pse_relations::generated::runtime::*;
        vec![
            computation_runs::RELATION_ID,
            modeling_checks::RELATION_ID,
            modeling_findings::RELATION_ID,
            modeling_reports::RELATION_ID,
            simulation_samples::RELATION_ID,
            simulation_events::RELATION_ID,
            response_sensitivities::RELATION_ID,
            modeling_trajectory_modes::RELATION_ID,
            candidate_assessments::RELATION_ID,
            trajectory_endpoints::RELATION_ID,
            scientific_contexts::RELATION_ID,
        ]
    }
    /// Advertised completion metadata without payload encoding.
    pub fn table_ids(&self) -> Vec<SemanticId> {
        Self::relation_ids()
    }
    /// Aggregate every advertised relation through its selected cursor.
    pub fn tables(
        &self,
    ) -> Result<Arc<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>>, WorkflowError>
    {
        let runtime = &self.inner.prepared.runtime;
        crate::workflow::result_export::collect_tables(self.table_ids(), runtime, |id| {
            self.cursor_by_id(id, 1024, crate::workflow::ResultOrder::Public)
                .map_err(Arc::new)
        })
        .map(Arc::new)
        .map_err(WorkflowError::Shared)
    }
    /// Materialize only the demanded relation.
    pub fn table(
        &self,
        name: &str,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let runtime = &self.inner.prepared.runtime;
        let id = runtime
            .registry()
            .relation(name)
            .ok_or_else(|| contract("unknown trajectory relation"))?
            .id;
        crate::workflow::result_export::collect(
            self.cursor_by_id(id, 1024, crate::workflow::ResultOrder::Public)?,
            runtime,
            id,
        )
        .map_err(WorkflowError::Shared)
    }
    /// Independent bounded traversal; escaped chunks retain their allocation owners.
    pub fn table_cursor(
        &self,
        name: &str,
        rows: usize,
    ) -> Result<crate::workflow::ResultCursor<'_>, WorkflowError> {
        let id = self
            .inner
            .prepared
            .runtime
            .registry()
            .relation(name)
            .ok_or_else(|| contract("unknown trajectory relation"))?
            .id;
        self.cursor_by_id(id, rows, crate::workflow::ResultOrder::Public)
    }
    /// Owning bounded selected transport for foreign-language streams.
    pub fn into_table_cursor(
        self: Arc<Self>,
        name: &str,
        rows: usize,
    ) -> Result<crate::workflow::ResultCursor<'static>, WorkflowError> {
        let runtime = &self.inner.prepared.runtime;
        let id = runtime
            .registry()
            .relation(name)
            .ok_or_else(|| contract("unknown result relation"))?
            .id;
        if !self.table_ids().contains(&id) {
            return Err(contract("relation is not part of result"));
        }
        if let Some(error) = self.inner.encodings.failure(id) {
            return Err(WorkflowError::Shared(error));
        }
        let spec = runtime
            .registry()
            .relation_by_id(id)
            .ok_or_else(|| contract("result declaration absent"))?;
        let schema = pse_schema::arrow::relation_schema_ref(runtime.registry(), spec)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        crate::workflow::ResultCursor::new(
            schema,
            id,
            rows,
            crate::workflow::ResultOrder::Public,
            |request| async move {
                self.project(&request)
                    .await
                    .map_err(|error| self.inner.encodings.record(id, error))
            },
        )
    }
    fn cursor_by_id(
        &self,
        id: SemanticId,
        rows: usize,
        order: crate::workflow::ResultOrder,
    ) -> Result<crate::workflow::ResultCursor<'_>, WorkflowError> {
        if !self.table_ids().contains(&id) {
            return Err(contract("relation is not part of trajectory"));
        }
        if let Some(error) = self.inner.encodings.failure(id) {
            return Err(WorkflowError::Shared(error));
        }
        let runtime = &self.inner.prepared.runtime;
        let spec = runtime
            .registry()
            .relation_by_id(id)
            .ok_or_else(|| contract("trajectory declaration absent"))?;
        let schema = pse_schema::arrow::relation_schema_ref(runtime.registry(), spec)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        crate::workflow::ResultCursor::new(schema, id, rows, order, |request| async move {
            self.project(&request)
                .await
                .map_err(|error| self.inner.encodings.record(id, error))
        })
    }
    pub(in crate::workflow) async fn project(
        &self,
        request: &crate::workflow::result_export::Projection,
    ) -> Result<(), WorkflowError> {
        if let Some(error) = self.inner.encodings.failure(request.relation) {
            return Err(WorkflowError::Shared(error));
        }
        self.project_impl(request).await.map_err(|error| {
            WorkflowError::Shared(self.inner.encodings.record(request.relation, error))
        })
    }
    async fn project_impl(
        &self,
        request: &crate::workflow::result_export::Projection,
    ) -> Result<(), WorkflowError> {
        use pse_relations::generated::runtime::{
            computation_runs, response_sensitivities, simulation_events, simulation_samples,
        };
        let p = &self.inner.prepared;
        let product = p.model().compiled();
        let pool = p.runtime.shared.pool();
        let cancel = request.cancel.clone();
        let validation = p.runtime.validation_context();
        let mut columns = crate::workflow::result_export::SelectedCollection::new(
            request,
            p.runtime.registry(),
            &pool,
            &cancel,
            &validation,
        );
        columns
            .ensure::<pse_model::generated::runtime::scientific_contexts::Row>()
            .map_err(relation)?;
        if request.wants(pse_relations::generated::runtime::scientific_contexts::RELATION_ID)
            && !columns.skip_next::<pse_model::generated::runtime::scientific_contexts::Row>()
        {
            let _copy = crate::workflow::result_export::working(
                &pool,
                "result:trajectory-context-copy",
                &[
                    (
                        p.coordinates.state.len()
                            + p.coordinates.parameters.len()
                            + p.contract.outputs.len()
                            + p.contract.quadratures.len()
                            + 1,
                        96,
                    ),
                    (
                        p.parameters.len()
                            + p.profile.samples.len()
                            + p.profile.atol.len()
                            + p.profile.out_atol.len(),
                        32,
                    ),
                    (p.modes.len(), 256),
                    (1, 65536),
                ],
            )?;
            columns
                .push(p.scientific_context(self.inner.run_id, self.inner.header.profile_identity)?)
                .await
                .map_err(relation)?;
        }
        columns
            .ensure::<computation_runs::Row>()
            .map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::modeling_checks::Row>()
            .map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::modeling_findings::Row>()
            .map_err(relation)?;
        if request.wants(pse_relations::generated::runtime::modeling_findings::RELATION_ID)
            && let Some(failure) = self.diagnostic()
        {
            columns
                .push(analysis_tables::finding_row(self.inner.run_id, 0, &failure))
                .await
                .map_err(relation)?;
        }
        if request.wants(pse_relations::generated::runtime::modeling_checks::RELATION_ID) {
            for check in &self.inner.checks {
                columns.push_ref(check).await.map_err(relation)?;
            }
        }
        columns
            .ensure::<pse_model::generated::runtime::modeling_reports::Row>()
            .map_err(relation)?;
        if request.wants(pse_relations::generated::runtime::modeling_reports::RELATION_ID) {
            for row in &self.inner.reports {
                columns.push_ref(row).await.map_err(relation)?;
            }
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
        if request.wants(pse_relations::generated::runtime::modeling_trajectory_modes::RELATION_ID)
        {
            for (sample, &(time, mode_index)) in
                self.inner.report.coverage.points.iter().enumerate()
            {
                if columns
                    .skip_next::<pse_model::generated::runtime::modeling_trajectory_modes::Row>()
                {
                    continue;
                }
                let mode = p
                    .modes
                    .get(mode_index)
                    .ok_or_else(|| contract("trajectory mode absent"))?;
                let _copy = crate::workflow::result_export::working(
                    &pool,
                    "result:trajectory-mode-copy",
                    &[(mode.name.len(), 1)],
                )?;
                columns
                    .push(
                        pse_model::generated::runtime::modeling_trajectory_modes::Row {
                            run_id: self.inner.run_id,
                            sample: sample as i64,
                            time,
                            mode: mode.name.clone(),
                        },
                    )
                    .await
                    .map_err(relation)?;
            }
        }
        if request.wants(pse_relations::generated::runtime::candidate_assessments::RELATION_ID) {
            columns
                .push_ref(&self.inner.assessment)
                .await
                .map_err(relation)?;
        }
        let r = &self.inner.report;
        columns
            .ensure::<pse_model::generated::runtime::trajectory_endpoints::Row>()
            .map_err(relation)?;
        if request.wants(pse_relations::generated::runtime::trajectory_endpoints::RELATION_ID)
            && !columns.skip_next::<pse_model::generated::runtime::trajectory_endpoints::Row>()
            && let Some(end) = &r.endpoint
        {
            let coverage = &self.inner.coverage;
            let _copy = crate::workflow::result_export::working(
                &pool,
                "result:trajectory-endpoint-copy",
                &[
                    (p.coordinates.state.len(), 24),
                    (p.coordinates.parameters.len(), 24),
                    (p.contract.outputs.len(), 16),
                    (end.point.outputs.len(), 8),
                    (end.point.integrals.len(), 8),
                    (end.input_columns.len(), 8),
                    (coverage.missing_observations.len(), 8),
                    (
                        p.modes
                            .get(end.point.mode)
                            .map_or(0, |mode| mode.name.len()),
                        1,
                    ),
                ],
            )?;
            columns
                .push(pse_model::generated::runtime::trajectory_endpoints::Row {
                    run_id: self.inner.run_id,
                    requirement: p.profile.endpoint.kind,
                    required_event: p.profile.endpoint.event,
                    event_id: end.event,
                    time: end.point.time,
                    mode: end.point.mode as i64,
                    mode_name: p
                        .modes
                        .get(end.point.mode)
                        .ok_or_else(|| contract("endpoint mode absent"))?
                        .name
                        .clone(),
                    mode_identity: p
                        .modes
                        .get(end.point.mode)
                        .ok_or_else(|| contract("endpoint mode absent"))?
                        .model
                        .compiled()
                        .admitted
                        .case()
                        .key(),
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
                .await
                .map_err(relation)?;
        }
        if request.wants(computation_runs::RELATION_ID) {
            columns
                .push_ref(&self.inner.header)
                .await
                .map_err(relation)?;
        }
        if request.wants(simulation_samples::RELATION_ID)
            || request.wants(response_sensitivities::RELATION_ID)
        {
            let _metadata = crate::workflow::result_export::working(
                &pool,
                "result:trajectory-coordinate-index",
                &[
                    (product.admitted.outputs.len(), 128),
                    (product.admitted.case().rows().len(), 128),
                    (p.contract.outputs.len(), 64),
                    (
                        if request.wants(response_sensitivities::RELATION_ID) {
                            product.admitted.case().parameters().len()
                                + product.admitted.case().variables().len()
                        } else {
                            0
                        },
                        144,
                    ),
                    (
                        if request.wants(response_sensitivities::RELATION_ID) {
                            p.contract
                                .parameters
                                .len()
                                .checked_add(r.forward_axes.len())
                                .ok_or_else(|| contract("trajectory sensitivity metadata extent"))?
                        } else {
                            0
                        },
                        size_of::<usize>(),
                    ),
                ],
            )?;
            let parameters = if request.wants(response_sensitivities::RELATION_ID) {
                let parameter_ports = product
                    .admitted
                    .case()
                    .parameters()
                    .iter()
                    .chain(product.admitted.case().variables().iter().map(|v| &v.port))
                    .collect::<Vec<_>>();
                let parameter_access =
                    pse_math::index::CheckedInventory::new(&parameter_ports, |v| v.id)
                        .map_err(math)?;
                let selected = sensitivity_parameters(
                    &p.profile,
                    p.contract.parameters.len(),
                    &r.forward_axes,
                )?;
                selected
                    .iter()
                    .zip(&r.forward_axes)
                    .map(|(parameter, axis)| {
                        let id =
                            p.contract.parameters.get(*parameter).ok_or_else(|| {
                                contract("trajectory sensitivity parameter absent")
                            })?;
                        parameter_access
                            .get(id)
                            .copied()
                            .map(|port| (*axis, port))
                            .ok_or_else(|| contract("trajectory parameter port absent"))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                Vec::new()
            };
            let output_access =
                pse_math::index::CheckedInventory::new(&product.admitted.outputs, |o| o.row_id())
                    .map_err(math)?;
            let row_access =
                pse_math::index::CheckedInventory::new(product.admitted.case().rows(), |r| r.id)
                    .map_err(math)?;
            let mut outputs = p
                .contract
                .outputs
                .iter()
                .enumerate()
                .map(|(column, id)| {
                    let output = output_access
                        .get(id)
                        .ok_or_else(|| contract("trajectory output lineage absent"))?;
                    let ModelingOutput::Member(symbol) = output else {
                        return Err(contract("trajectory output is not an authored member"));
                    };
                    let row = row_access
                        .get(id)
                        .ok_or_else(|| contract("trajectory physical row absent"))?;
                    let unit = p
                        .quantities
                        .quantity_type(row.quantity)
                        .map_err(math)?
                        .canonical_unit;
                    Ok((*symbol, row.quantity.as_id(), unit.as_id(), column))
                })
                .collect::<Result<Vec<_>, WorkflowError>>()?;
            if request.order == crate::workflow::ResultOrder::Canonical
                && request.wants(simulation_samples::RELATION_ID)
            {
                outputs.sort_unstable_by_key(|(symbol, _, _, _)| *symbol);
            }
            let samples = if r.coverage.retention == native::SampleRetention::Full {
                r.samples.as_slice()
            } else {
                &[]
            };
            let width = outputs.len();
            let total = samples
                .len()
                .checked_mul(width)
                .ok_or_else(|| contract("trajectory projection extent"))?;
            let start = request.range.start.min(total);
            let end = request.range.end.min(total);
            if request.wants(simulation_samples::RELATION_ID) {
                columns.set_position(start);
            }
            for index in if request.wants(simulation_samples::RELATION_ID) {
                start..end
            } else {
                0..total
            } {
                let order = if request.wants(simulation_samples::RELATION_ID) {
                    request.order
                } else {
                    crate::workflow::ResultOrder::Public
                };
                let (sample, i) = order.trajectory_coordinate(index, width, samples.len())?;
                let point = &samples[sample];
                let (symbol, quantity, unit, column) = outputs[i];
                if request.wants(simulation_samples::RELATION_ID) {
                    if point.outputs.len() != width {
                        return Err(contract("trajectory output extent"));
                    }
                    columns
                        .push(simulation_samples::Row {
                            run_id: self.inner.run_id,
                            sample: sample as i64,
                            time: point.time,
                            symbol_id: symbol,
                            quantity_id: quantity,
                            unit_id: unit,
                            value: point.outputs[column],
                        })
                        .await
                        .map_err(relation)?;
                } else if !point.output_sensitivities.is_empty() {
                    let extent = width
                        .checked_mul(parameters.len())
                        .ok_or_else(|| contract("trajectory sensitivity extent overflow"))?;
                    if point.output_sensitivities.len() != extent {
                        return Err(contract("trajectory sensitivity extent"));
                    }
                    for (j, (axis, parameter)) in parameters.iter().enumerate() {
                        if columns.skip_next::<response_sensitivities::Row>() {
                            continue;
                        }
                        columns
                            .push(response_sensitivities::Row {
                                run_id: self.inner.run_id,
                                experiment_id: p.solved.instance(),
                                sample: sample as i64,
                                time: Some(point.time),
                                output_id: symbol,
                                parameter_id: parameter.id,
                                parameter_column: i64::try_from(*axis).map_err(|_| {
                                    contract("trajectory physical sensitivity column extent")
                                })?,
                                output_unit_id: unit,
                                parameter_unit_id: parameter.unit.as_id(),
                                value: point.output_sensitivities[column * parameters.len() + j],
                            })
                            .await
                            .map_err(relation)?;
                    }
                }
            }
        }
        if request.wants(simulation_events::RELATION_ID) {
            for (ordinal, event) in r.events.iter().enumerate() {
                for (i, state) in p.coordinates.state.iter().enumerate() {
                    if columns.skip_next::<simulation_events::Row>() {
                        continue;
                    }
                    if event.before.len() != p.coordinates.state.len()
                        || event
                            .after
                            .as_ref()
                            .is_some_and(|v| v.len() != p.coordinates.state.len())
                    {
                        return Err(contract("trajectory event extent"));
                    }
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
                        .await
                        .map_err(relation)?;
                }
            }
        }
        columns.finish().await.map_err(relation)
    }
}

impl ModelingSimulation {
    /// Check complete original identity ambiguity once, then retain only selected
    /// physical lineage in native output order. All preparation scratch is prepaid
    /// and every inventory/projection step checks the original run stop scope.
    pub(in crate::workflow) fn live_projection(
        &self,
        cancel: &pse_diagnostics::Cancellation,
        deadline: std::time::Instant,
    ) -> Result<crate::workflow::live_simulation::LiveProjection, WorkflowError> {
        use crate::workflow::live_simulation::{LiveOutputLineage, LiveProjection, encoding_checkpoint};
        encoding_checkpoint(cancel, deadline)?;
        let product = self.model().compiled();
        let selected = self.contract.outputs.len();
        let scratch_bytes = product.admitted.outputs.len()
            .checked_add(product.admitted.case().rows().len())
            .and_then(|count| count.checked_add(selected.checked_mul(4)?))
            .and_then(|count| count.checked_mul(size_of::<SemanticId>() + size_of::<usize>() + 256))
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or_else(|| contract("live projection scratch extent"))?;
        let _scratch = self.runtime.native().reserve("simulation:live-projection-scratch", scratch_bytes)?;
        let retained_bytes = selected.checked_mul(size_of::<LiveOutputLineage>())
            .and_then(|bytes| bytes.checked_add(self.modes.len().checked_mul(size_of::<ContentHash>())?))
            .and_then(|bytes| bytes.checked_add(size_of::<LiveProjection>() + 2 * size_of::<usize>()))
            .ok_or_else(|| contract("live projection retained extent"))?;
        let owner = self.runtime.native().reserve("simulation:live-projection", retained_bytes)?;
        let mut positions = BTreeMap::new();
        for (position, id) in self.contract.outputs.iter().enumerate() {
            encoding_checkpoint(cancel, deadline)?;
            if positions.insert(*id, position).is_some() { return Err(contract("live trajectory selected output repeats")); }
        }
        let mut symbols = vec![None; selected];
        let mut seen = BTreeSet::new();
        for output in &product.admitted.outputs {
            encoding_checkpoint(cancel, deadline)?;
            let id = output.row_id();
            if !seen.insert(id) { return Err(math(pse_math::MathError::Contract("duplicate projection source identity".into()))); }
            if let Some(position) = positions.get(&id) {
                let ModelingOutput::Member(symbol) = output else { return Err(contract("trajectory output is not an authored member")); };
                symbols[*position] = Some(*symbol);
            }
        }
        for symbol in &symbols {
            encoding_checkpoint(cancel, deadline)?;
            if symbol.is_none() { return Err(contract("trajectory output lineage absent")); }
        }
        seen.clear();
        let mut lineage = vec![None; selected];
        for row in product.admitted.case().rows() {
            encoding_checkpoint(cancel, deadline)?;
            if !seen.insert(row.id) { return Err(math(pse_math::MathError::Contract("duplicate projection source identity".into()))); }
            if let Some(position) = positions.get(&row.id) {
                let unit = self.quantities.quantity_type(row.quantity).map_err(math)?.canonical_unit;
                lineage[*position] = Some(LiveOutputLineage {
                    symbol: symbols[*position].ok_or_else(|| contract("trajectory output lineage absent"))?,
                    quantity: row.quantity.as_id(), unit: unit.as_id(),
                });
            }
        }
        let mut outputs = Vec::new();
        outputs.try_reserve_exact(selected).map_err(|_| contract("live projection allocation refused"))?;
        for entry in lineage {
            encoding_checkpoint(cancel, deadline)?;
            outputs.push(entry.ok_or_else(|| contract("trajectory physical row absent"))?);
        }
        let mut modes = Vec::new();
        modes.try_reserve_exact(self.modes.len()).map_err(|_| contract("live mode projection allocation refused"))?;
        for (ordinal, mode) in self.modes.iter().enumerate() {
            encoding_checkpoint(cancel, deadline)?;
            i64::try_from(ordinal).map_err(|_| contract("live trajectory mode ordinal extent"))?;
            modes.push(mode.model.compiled().admitted.case().key());
        }
        encoding_checkpoint(cancel, deadline)?;
        Ok(LiveProjection { outputs, modes, source: product.admitted.clone(), _owner: owner })
    }
}

#[cfg(test)]
mod sensitivity_coordinate_tests {
    use super::*;
    #[test]
    fn selected_public_parameters_follow_schedule_order_and_physical_axis_identity() {
        let profile = native::Profile {
            schedule: vec![
                native::ScheduledInput {
                    parameter: 2,
                    times: vec![0.25, 0.75],
                },
                native::ScheduledInput {
                    parameter: 0,
                    times: vec![0.5],
                },
            ],
            ..Default::default()
        };
        assert_eq!(
            sensitivity_parameters(&profile, 3, &[5, 0, 2, 4]).unwrap(),
            vec![0, 1, 2, 0]
        );
        assert!(sensitivity_parameters(&profile, 3, &[]).unwrap().is_empty());
        for axes in [vec![0, 0], vec![6], vec![usize::MAX]] {
            assert!(sensitivity_parameters(&profile, 3, &axes).is_err());
        }
    }
}
