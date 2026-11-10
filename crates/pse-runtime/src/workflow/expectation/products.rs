// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Registered original products are projected once by their producing run, before sealing.
use super::*;
use crate::workflow::result_export::{Projection, Rows};
use crate::workflow::{RunReport, RunResult, WorkflowError, contract, relation};
use pse_model::generated::{
    identities::{RunId, StudyId},
    structures::{ExpectationOriginalOutput, ExpectationPhysicalInput},
};
use pse_relations::generated::runtime::{
    expectation_estimates as estimates, expectation_samples as samples,
    expectation_summaries as summaries,
};
use std::sync::Arc;

fn ordinal(value: usize) -> Result<i64, WorkflowError> {
    i64::try_from(value).map_err(|_| contract("expectation result ordinal"))
}
/// One immutable decoded iteration, shared by every row in this projection. No per-page reparse.
struct OriginalIteration {
    parent: StudyId,
    child: StudyId,
    iteration: usize,
    points: Vec<ExpectationSample>,
    observed: Vec<MemberObservation>,
    definition: crate::workflow::StudyDefinition,
    outputs: Vec<pse_ids::SemanticId>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl OriginalIteration {
    async fn read(product: &ExpectationProduct, iteration: usize) -> Result<Self, WorkflowError> {
        let preparation = product.runtime.preparation_services();
        let owner = preparation.math().reserve(
            "expectation:original-iteration-projection",
            8 * execution::DOCUMENT_BYTES,
        )?;
        let store = product.runtime.canonical_store()?;
        let proposal = store
            .expectation_proposal(&product.parent.to_string(), iteration as u64)
            .await?
            .ok_or_else(|| contract("expectation original proposal absent"))?;
        let boundary = store
            .expectation_boundary_at(&product.parent.to_string(), iteration as u64)
            .await?
            .ok_or_else(|| contract("expectation original child has no settled boundary"))?;
        let points: Vec<ExpectationSample> = execution::decode(proposal.samples.as_slice())?;
        let observed: Vec<MemberObservation> = execution::decode(boundary.observations.as_slice())?;
        let definition: crate::workflow::StudyDefinition =
            execution::decode(proposal.definition.as_slice())?;
        if points.len() != observed.len()
            || definition.points.len() != points.len()
            || boundary.child != proposal.child
        {
            return Err(contract(
                "expectation original iteration inventories differ",
            ));
        }
        let child = StudyId::from_id(
            pse_ids::SemanticId::parse_hex(&proposal.child)
                .map_err(|error| contract(error.to_string()))?,
        );
        Ok(Self {
            parent: product.parent,
            child,
            iteration,
            points,
            observed,
            definition,
            outputs: product
                .report
                .request
                .outputs
                .iter()
                .map(|output| output.output)
                .collect(),
            _owner: owner,
        })
    }
    fn row(&self, index: usize, run_id: RunId) -> Result<samples::Row, WorkflowError> {
        let point = &self.points[index];
        let observation = &self.observed[index];
        if point.ordinal != observation.ordinal
            || observation.child != self.child
            || observation.values.len() != self.outputs.len()
        {
            return Err(contract(
                "expectation original point correspondence differs",
            ));
        }
        let inputs = self.definition.points[index]
            .binding
            .entries
            .values()
            .map(|entry| ExpectationPhysicalInput {
                member: entry.member,
                quantity: entry.quantity,
                canonical: entry.canonical.into_inner(),
            })
            .collect();
        let outputs = self
            .outputs
            .iter()
            .zip(&observation.values)
            .map(|(member, canonical)| ExpectationOriginalOutput {
                member: *member,
                canonical: *canonical,
            })
            .collect();
        Ok(samples::Row {
            run_id,
            parent_id: self.parent,
            child_id: self.child,
            iteration: ordinal(self.iteration)?,
            ordinal: i64::try_from(point.ordinal)
                .map_err(|_| contract("expectation sample ordinal"))?,
            point_index: ordinal(index)?,
            lane: ordinal(point.lane)?,
            lane_ordinal: ordinal(point.lane_ordinal)?,
            target_mass: point.target_mass,
            proposal_weight: point.proposal_weight,
            categories: point.categories.clone(),
            base_coordinates: point.base.clone(),
            inputs,
            outputs,
            scientific_failure: observation.scientific_failure,
            operational_incomplete: observation.operational_incomplete,
        })
    }
}
impl RunResult {
    pub(in crate::workflow) async fn project_expectation(
        &self,
        projection: &Projection,
    ) -> Result<(), WorkflowError> {
        let preparation = self.services.preparation();
        let pool = preparation.pool();
        let validation = self.result_validation_context();
        let product = match self.report() {
            Ok(RunReport::Expectation(product)) => Some(product),
            _ => None,
        };
        if projection.wants(summaries::RELATION_ID) {
            let mut rows = Rows::<summaries::Row>::new(
                projection,
                preparation.registry(),
                &pool,
                &projection.cancel,
                &validation,
            )
            .map_err(relation)?;
            if let Some(product) = product {
                let report = &product.report;
                rows.push(summaries::Row {
                    run_id: self.run_id,
                    parent_id: product.parent,
                    source_revision: report.request.source.revision.as_id(),
                    physical_context: report.request.source.physical_context,
                    completed_iterations: ordinal(report.completed_iterations)?,
                    recorded_iterations: ordinal(product.recorded_iterations)?,
                    issued: ordinal(report.issued)?,
                    interrupted: report.interrupted,
                    limit_reached: report.limit_reached,
                    failure_probability: report
                        .failure_probability
                        .as_ref()
                        .map(|value| value.estimate),
                    failure_standard_error: report
                        .failure_probability
                        .as_ref()
                        .map(|value| value.standard_error),
                    failure_probability_complete: report.failure_probability_complete,
                    original_report: execution::report_document(report)?,
                })
                .await
                .map_err(relation)?;
            }
            return rows.finish().await.map_err(relation);
        }
        if projection.wants(estimates::RELATION_ID) {
            let mut rows = Rows::<estimates::Row>::new(
                projection,
                preparation.registry(),
                &pool,
                &projection.cancel,
                &validation,
            )
            .map_err(relation)?;
            if let Some(product) = product {
                let report = &product.report;
                for output in &report.outputs {
                    rows.push(estimates::Row {
                        run_id: self.run_id,
                        parent_id: product.parent,
                        output_id: output.output,
                        quantity_id: output.quantity,
                        unit_id: output.unit,
                        completed_iterations: ordinal(report.completed_iterations)?,
                        issued: ordinal(output.issued)?,
                        available: ordinal(output.available)?,
                        unavailable_ordinals: output
                            .unavailable
                            .iter()
                            .map(|value| {
                                i64::try_from(*value)
                                    .map_err(|_| contract("expectation unavailable ordinal"))
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                        reference: output.reference,
                        estimate: output.estimate,
                        standard_error: output.standard_error,
                        complete: output.complete,
                        precision_met: output.precision_met,
                        interrupted: report.interrupted,
                        limit_reached: report.limit_reached,
                    })
                    .await
                    .map_err(relation)?;
                }
            }
            return rows.finish().await.map_err(relation);
        }
        if projection.wants(samples::RELATION_ID) {
            let mut rows = Rows::<samples::Row>::new(
                projection,
                preparation.registry(),
                &pool,
                &projection.cancel,
                &validation,
            )
            .map_err(relation)?;
            if let Some(product) = product {
                for iteration in 0..product.recorded_iterations {
                    let original = OriginalIteration::read(product, iteration).await?;
                    for index in 0..original.points.len() {
                        rows.push(original.row(index, self.run_id)?)
                            .await
                            .map_err(relation)?;
                    }
                }
            }
            return rows.finish().await.map_err(relation);
        }
        Err(contract("relation is not an original expectation product"))
    }
}
