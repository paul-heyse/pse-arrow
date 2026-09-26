// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical trajectories and exact execution provenance use generated owned relations.
use super::{RunReport, RunRequest, RunResult, WorkflowError, contract, relation};
use pse_ids::SemanticId;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{
        authored::computation_models,
        runtime::{
            computation_runs, response_sensitivities, simulation_events, simulation_samples,
            solve_metrics,
        },
    },
};
use std::collections::BTreeMap;
impl RunResult {
    pub(super) fn encode_simulation(
        &self,
    ) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
        let RunRequest::Simulation(p) = &self.request else {
            return Err(contract("simulation request mismatch"));
        };
        let report = match &self.report {
            Ok(RunReport::Simulation(r)) => Some(r),
            Err(_) => None,
            _ => return Err(contract("simulation report mismatch")),
        };
        let registry = &self.runtime.registry;
        let staging = pse_columnar::MemoryConsumer::new("workflow:simulation-encoding")
            .register(&self.runtime.shared.pool());
        staging
            .try_grow(
                p.bytes
                    .checked_mul(4)
                    .ok_or_else(|| contract("simulation encoding extent overflow"))?,
            )
            .map_err(|e| WorkflowError::Math(e.into()))?;
        let mut header = computation_runs::Builder::with_registry(registry, 1).map_err(relation)?;
        header
            .push(
                self.completion()
                    .map_err(|e| contract(e.to_string()))?
                    .computation
                    .clone()
                    .ok_or_else(|| contract("missing completed computation"))?,
            )
            .map_err(relation)?;
        let mut samples =
            simulation_samples::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut events =
            simulation_events::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut sensitivities =
            response_sensitivities::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut metrics = solve_metrics::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut metric = |namespace: &str, name: &str, value: Metric| {
            super::results::push_metric(&mut metrics, self.run_id, 0, namespace, name, &value)
        };
        use pse_backend_native::solve::Metric;
        metric(
            "profile",
            "effective",
            Metric::Text(pse_backend_native::dynamics::profile_json(&p.profile).to_string()),
        )?;
        metric(
            "provenance",
            "build.identity",
            Metric::Text(pse_buildinfo::BUILD_IDENTITY.to_prefixed()),
        )?;
        metric(
            "provenance",
            "prepared.identity",
            Metric::Text(p.key.to_prefixed()),
        )?;
        if let Some(r) = report {
            metric("result", "completed_time", Metric::Real(r.completed_time))?;
            metric(
                "progress",
                "dropped",
                Metric::Integer(r.dropped_progress.min(i64::MAX as u64) as i64),
            )?;
            for (i, e) in r.progress.iter().enumerate() {
                let ns = format!("progress.{i}.{}", e.phase);
                metric(
                    &ns,
                    "elapsed_seconds",
                    Metric::Real(e.elapsed.as_secs_f64()),
                )?;
                for (name, value) in &e.values {
                    metric(&ns, name, value.clone())?;
                }
            }
            for (i, s) in r.statistics.iter().enumerate() {
                metric(
                    "diffsol.statistics",
                    &i.to_string(),
                    Metric::Text(s.to_string()),
                )?;
            }
            for (kind, values) in [
                ("requested_initial", &r.requested_initial),
                ("consistent_initial", &r.consistent_initial),
            ] {
                for (i, &value) in values.iter().enumerate() {
                    let state = &p.declaration.states[i];
                    metric(
                        kind,
                        &state.symbol_id.to_hex(),
                        Metric::Real(value * state.scale + state.offset),
                    )?;
                }
            }
            for (ordinal, s) in r.samples.iter().enumerate() {
                for (i, port) in p
                    .state_ports
                    .iter()
                    .chain(p.output_ports.iter())
                    .enumerate()
                {
                    let value = if i < p.state_ports.len() {
                        s.state[i] * p.declaration.states[i].scale + p.declaration.states[i].offset
                    } else {
                        s.outputs[i - p.state_ports.len()]
                    };
                    let unit = p
                        .revision
                        .0
                        .physical
                        .quantities
                        .quantity_type(port.quantity)
                        .map_err(super::math)?
                        .canonical_unit;
                    samples
                        .push(simulation_samples::Row {
                            run_id: self.run_id,
                            sample: ordinal as i64,
                            time: s.time,
                            symbol_id: port.id,
                            quantity_id: port.quantity.as_id(),
                            unit_id: unit.as_id(),
                            value,
                        })
                        .map_err(relation)?;
                    let derivatives = if i < p.state_ports.len() {
                        (!s.state_sensitivities.is_empty()).then_some((
                            &s.state_sensitivities,
                            i,
                            p.declaration.states[i].scale,
                        ))
                    } else {
                        (!s.output_sensitivities.is_empty()).then_some((
                            &s.output_sensitivities,
                            i - p.state_ports.len(),
                            1.0,
                        ))
                    };
                    if let Some((values, row, scale)) = derivatives {
                        for (j, param) in p.parameter_ports.iter().enumerate() {
                            let punit = p
                                .revision
                                .0
                                .physical
                                .quantities
                                .quantity_type(param.quantity)
                                .map_err(super::math)?
                                .canonical_unit;
                            sensitivities
                                .push(response_sensitivities::Row {
                                    run_id: self.run_id,
                                    experiment_id: p.declaration.dynamic_id,
                                    sample: ordinal as i64,
                                    time: Some(s.time),
                                    output_id: port.id,
                                    parameter_id: param.id,
                                    output_unit_id: unit.as_id(),
                                    parameter_unit_id: punit.as_id(),
                                    value: values[row * p.parameter_ports.len() + j] * scale,
                                })
                                .map_err(relation)?;
                        }
                    }
                }
            }
            for (i, e) in r.events.iter().enumerate() {
                for (j, s) in p.declaration.states.iter().enumerate() {
                    events
                        .push(simulation_events::Row {
                            run_id: self.run_id,
                            ordinal: i as i64,
                            event_id: e.event,
                            time: e.time,
                            symbol_id: s.symbol_id,
                            before: e.before[j] * s.scale + s.offset,
                            after: e.after.as_ref().map(|v| v[j] * s.scale + s.offset),
                        })
                        .map_err(relation)?;
                }
            }
        }
        let mut model =
            computation_models::Builder::with_registry(registry, 1).map_err(relation)?;
        model.push(p.revision.0.row.clone()).map_err(relation)?;
        let mut batches = BTreeMap::from([
            (
                computation_runs::RELATION_ID,
                header.finish().map_err(relation)?,
            ),
            (
                simulation_samples::RELATION_ID,
                samples.finish().map_err(relation)?,
            ),
            (
                simulation_events::RELATION_ID,
                events.finish().map_err(relation)?,
            ),
            (
                response_sensitivities::RELATION_ID,
                sensitivities.finish().map_err(relation)?,
            ),
            (
                solve_metrics::RELATION_ID,
                metrics.finish().map_err(relation)?,
            ),
            (
                computation_models::RELATION_ID,
                model.finish().map_err(relation)?,
            ),
        ]);
        self.retain_sources(&mut batches)?;
        Ok(batches)
    }
    pub(super) fn retain_sources(
        &self,
        batches: &mut BTreeMap<SemanticId, FieldCheckedBatch>,
    ) -> Result<(), WorkflowError> {
        self.numerical_tables(batches)?;
        use pse_relations::generated::runtime::run_lineage;
        let completion = self.completion().map_err(|e| contract(e.to_string()))?;
        let mut lineage =
            run_lineage::Builder::with_registry(&self.runtime.registry, completion.lineage.len())
                .map_err(relation)?;
        for row in &completion.lineage {
            lineage.push(row.clone()).map_err(relation)?;
        }
        batches.insert(
            run_lineage::RELATION_ID,
            lineage.finish().map_err(relation)?,
        );
        let mut sources = super::SourceDeclarations::default();
        for revision in self.request.revisions() {
            sources.merge(&revision.0.sources)?;
        }
        batches.extend(sources.tables(&self.runtime.registry)?);
        let cancel = pse_columnar::CancellationToken::new();
        for batch in batches.values_mut() {
            *batch = batch
                .retained(&self.runtime.shared.pool(), &cancel)
                .map_err(relation)?;
        }
        for revision in self.request.revisions() {
            for (key, batch) in &revision.0.physical.sources {
                let id = self
                    .runtime
                    .registry
                    .relation(&key.qualified_name())
                    .ok_or_else(|| contract("physical source relation absent"))?
                    .id;
                if let Some(existing) = batches.get(&id) {
                    if existing.batch() != batch.batch() {
                        return Err(contract(
                            "run mixes incompatible physical source inventories",
                        ));
                    }
                } else {
                    batches.insert(id, batch.clone());
                }
            }
        }
        Ok(())
    }
}
