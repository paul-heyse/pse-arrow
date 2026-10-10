// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected generated trajectory projection, preserving exact physical coordinates.
use super::*;
use crate::workflow::math;
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
            .registry
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
            .registry
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
            .registry
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
            .registry
            .relation_by_id(id)
            .ok_or_else(|| contract("result declaration absent"))?;
        let schema = pse_schema::arrow::relation_schema_ref(&runtime.registry, spec)
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
            .registry
            .relation_by_id(id)
            .ok_or_else(|| contract("trajectory declaration absent"))?;
        let schema = pse_schema::arrow::relation_schema_ref(&runtime.registry, spec)
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
        let validation = p.runtime.validation_context()?;
        let mut columns = crate::workflow::result_export::SelectedCollection::new(
            request,
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
            for (sample, point) in self.inner.report.samples.iter().enumerate() {
                if columns
                    .skip_next::<pse_model::generated::runtime::modeling_trajectory_modes::Row>()
                {
                    continue;
                }
                let mode = p
                    .modes
                    .get(point.mode)
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
                            time: point.time,
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
                            p.contract.parameters.len()
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
                p.contract
                    .parameters
                    .iter()
                    .map(|id| {
                        parameter_access
                            .get(id)
                            .copied()
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
            let width = outputs.len();
            let total = r
                .samples
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
                let (sample, i) = order.trajectory_coordinate(index, width, r.samples.len())?;
                let point = &r.samples[sample];
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
                    for (j, parameter) in parameters.iter().enumerate() {
                        if columns.skip_next::<response_sensitivities::Row>() {
                            continue;
                        }
                        if point.output_sensitivities.len() != width * parameters.len() {
                            return Err(contract("trajectory sensitivity extent"));
                        }
                        columns
                            .push(response_sensitivities::Row {
                                run_id: self.inner.run_id,
                                experiment_id: p.solved.instance(),
                                sample: sample as i64,
                                time: Some(point.time),
                                output_id: symbol,
                                parameter_id: parameter.id,
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
