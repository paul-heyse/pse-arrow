// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated physical trajectory transport, including interrupted native outcomes.
use super::*;

impl ModelingTrajectory {
    /// Columns retain pool ownership after all source and trajectory handles are dropped.
    pub fn tables(
        &self,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        use pse_model::generated::{
            enums::{ComputationKind, NativeBackend, NativeQualification, NativeRunState},
            runtime::{
                computation_runs, response_sensitivities, simulation_events, simulation_samples,
            },
        };
        let p = &self.prepared;
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
                        self.reports
                            .iter()
                            .map(pse_model::HeapUsage::owned_bytes)
                            .max()
                            .unwrap_or(0),
                    )
                })
                .ok_or_else(|| contract("trajectory row scratch extent"))?,
        )?;
        let mut columns =
            pse_relations::columnar::Collection::new(&p.runtime.registry, &pool, &cancel);
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
                .push(super::super::analysis_tables::finding_row(
                    self.run_id,
                    0,
                    &failure,
                ))
                .map_err(relation)?;
        }
        for check in &self.checks {
            columns.push(check.clone()).map_err(relation)?;
        }
        columns
            .ensure::<pse_model::generated::runtime::modeling_reports::Row>()
            .map_err(relation)?;
        for row in &self.reports {
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
        for (sample, point) in self.report.samples.iter().enumerate() {
            let mode = p
                .modes
                .get(point.mode)
                .ok_or_else(|| contract("trajectory mode absent"))?;
            columns
                .push(
                    pse_model::generated::runtime::modeling_trajectory_modes::Row {
                        run_id: self.run_id,
                        sample: sample as i64,
                        time: point.time,
                        mode: mode.name.clone(),
                    },
                )
                .map_err(relation)?;
        }
        let r = &self.report;
        let error = r.error.as_ref().map(super::super::engines::bounded_error);
        columns
            .push(computation_runs::Row {
                run_id: self.run_id,
                kind: ComputationKind::Simulation,
                source_identity: p.source.revision.identity(),
                profile_identity: super::super::super::dynamics::profile_identity(&p.profile),
                state: NativeRunState::Native,
                termination: None,
                trajectory_termination: Some(r.termination),
                backend: match p.profile.method {
                    native::Method::Diffsol => Some(NativeBackend::Diffsol),
                    native::Method::Idas => Some(NativeBackend::Idas),
                    native::Method::Auto => None,
                },
                native_code: None,
                native_status: None,
                qualification: if self.accepted {
                    NativeQualification::Feasible
                } else {
                    NativeQualification::Unqualified
                },
                candidate_kind: None,
                candidate_available: !r.samples.is_empty(),
                feasible: self
                    .checks_complete
                    .then_some(self.checks.iter().all(|c| c.satisfied)),
                completed_time: Some(r.completed_time),
                completed_samples: Some(r.samples.len() as i64),
                estimate_qualified: None,
                response_available: Some(
                    r.samples.iter().any(|s| !s.output_sensitivities.is_empty()),
                ),
                response_rank: None,
                response_condition: None,
                validation_error: self.validation_error.as_ref().map(ToString::to_string),
                error,
            })
            .map_err(relation)?;
        let parameters = p
            .contract
            .parameters
            .iter()
            .map(|id| {
                product
                    .admitted
                    .case
                    .parameters()
                    .iter()
                    .chain(product.admitted.case.variables().iter().map(|v| &v.port))
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
                    .case
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
                        run_id: self.run_id,
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
                                run_id: self.run_id,
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
                        run_id: self.run_id,
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
