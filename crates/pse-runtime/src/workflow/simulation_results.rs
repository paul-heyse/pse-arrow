// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical results reuse one relation projection for convenience and durable transport.
#[cfg(feature = "solver-diffsol")]
use super::RunRequest;
use super::{RunReport, RunResult, WorkflowError, contract, relation};
use pse_relations::generated::runtime::computation_runs;
impl RunResult {
    pub(super) async fn encode_simulation(
        &self,
        request: &super::result_export::Projection,
    ) -> Result<(), WorkflowError> {
        if let Ok(RunReport::Simulation(trajectory)) = &self.report {
            return trajectory.project(request).await;
        }
        let pool = self.runtime.shared.pool();
        let cancel = request.cancel.clone();
        let validation = self.runtime.validation_context()?;
        let mut header = super::result_export::Rows::<computation_runs::Row>::new(
            request,
            &self.runtime.registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        if request.wants(computation_runs::RELATION_ID) {
            header
                .push(
                    self.completion()
                        .map_err(|e| contract(e.to_string()))?
                        .computation
                        .clone()
                        .ok_or_else(|| contract("simulation completion absent"))?,
                )
                .await
                .map_err(relation)?;
        }
        header.finish().await.map_err(relation)
    }
    #[cfg(feature = "solver-diffsol")]
    pub(super) async fn encode_shooting(
        &self,
        request: &super::result_export::Projection,
    ) -> Result<(), WorkflowError> {
        let RunRequest::Shooting { problem, .. } = &self.request else {
            return Err(contract("shooting request mismatch"));
        };
        let validation = self.runtime.validation_context()?;
        let report = match &self.report {
            Ok(RunReport::Shooting(report)) => Some(report.as_ref()),
            _ => None,
        };
        let registry = &self.runtime.registry;
        let pool = self.runtime.shared.pool();
        let cancel = request.cancel.clone();
        if super::ModelingTrajectory::relation_ids().contains(&request.relation)
            && let Some(report) = report.filter(|r| r.trajectory.is_some())
        {
            let owner = self
                ._owner
                .clone()
                .ok_or_else(|| contract("shooting result ownership absent"))?;
            let header = self
                .completion()
                .map_err(|e| contract(e.to_string()))?
                .computation
                .clone()
                .ok_or_else(|| contract("shooting completion absent"))?;
            problem
                .simulation
                .completed_trajectory(
                    report,
                    header,
                    self.assessments
                        .first()
                        .cloned()
                        .ok_or_else(|| contract("shooting assessment absent"))?,
                    owner,
                )?
                .project(request)
                .await?;
            return Ok(());
        }
        use pse_model::generated::enums::{
            DualQualification, ModelingVariableDomain, NumericalTarget,
        };
        use pse_relations::generated::runtime::{
            solve_constraints, solve_metrics, solve_variables,
        };
        let native = report.and_then(|r| r.solve.as_ref());
        let candidate = report.and_then(|r| r.candidate.as_ref());
        let policy = problem.numerics();
        let metadata = |id, kind| {
            policy
                .targets
                .iter()
                .find(|t| t.id == id && t.kind == kind)
                .ok_or_else(|| contract("shooting coordinate numerical metadata absent"))
        };
        let mut variables = crate::workflow::result_export::Rows::<solve_variables::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        if request.wants(solve_variables::RELATION_ID) {
            for (i, variable) in problem.contract().variables.iter().enumerate() {
                if variables.skip_next() {
                    continue;
                }
                let target = metadata(variable.id, NumericalTarget::Variable)?;
                variables
                    .push(solve_variables::Row {
                        run_id: self.run_id,
                        step: 0,
                        symbol_id: variable.id,
                        quantity_id: Some(target.quantity),
                        unit_id: Some(target.unit),
                        fixed: false,
                        parameter: false,
                        domain: Some(ModelingVariableDomain::Continuous),
                        value: candidate.and_then(|c| c.get(i).copied()),
                        lower: variable.lower.is_finite().then_some(variable.lower),
                        upper: variable.upper.is_finite().then_some(variable.upper),
                        lower_violation: None,
                        upper_violation: None,
                        tolerance: Some(problem.tolerances().variables[i]),
                        lower_dual: None,
                        upper_dual: None,
                        reduced_cost: None,
                        stationarity: None,
                        dual_qualification: DualQualification::Unavailable,
                    })
                    .await
                    .map_err(relation)?;
            }
        }

        variables.finish().await.map_err(relation)?;
        let mut constraints = crate::workflow::result_export::Rows::<solve_constraints::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        if request.wants(solve_constraints::RELATION_ID) {
            for (i, id) in problem.contract().rows.iter().enumerate() {
                if constraints.skip_next() {
                    continue;
                }
                let target = metadata(*id, NumericalTarget::Row)?;
                let (lower, upper) = problem.constraint_bounds()[i];
                constraints
                    .push(solve_constraints::Row {
                        run_id: self.run_id,
                        step: 0,
                        row_id: *id,
                        quantity_id: Some(target.quantity),
                        unit_id: Some(target.unit),
                        value: report
                            .and_then(|r| r.constraint_values.as_ref())
                            .and_then(|v| v.get(i).copied()),
                        lower: lower.is_finite().then_some(lower),
                        upper: upper.is_finite().then_some(upper),
                        equality_residual: None,
                        lower_violation: None,
                        upper_violation: None,
                        tolerance: Some(problem.tolerances().rows[i]),
                        dual: None,
                        dual_qualification: DualQualification::Unavailable,
                    })
                    .await
                    .map_err(relation)?;
            }
        }

        constraints.finish().await.map_err(relation)?;
        let mut metrics = crate::workflow::result_export::Rows::<solve_metrics::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        if let Some(native) = native {
            super::results::push_native_metrics(&mut metrics, self.run_id, 0, native).await?;
        }
        if let Some(report) = report {
            for (name, value) in [
                ("objective", report.objective),
                ("continuity", report.continuity),
            ] {
                if let Some(value) = value {
                    super::results::push_metric(
                        &mut metrics,
                        self.run_id,
                        0,
                        "shooting",
                        name,
                        &pse_backend_native::solve::Metric::Real(value),
                    )
                    .await?;
                }
            }
        }
        metrics.finish().await.map_err(relation)?;
        if request.wants(computation_runs::RELATION_ID) {
            let mut header = crate::workflow::result_export::Rows::<computation_runs::Row>::new(
                request,
                registry,
                &pool,
                &cancel,
                &validation,
            )
            .map_err(relation)?;
            header
                .push(
                    self.completion()
                        .map_err(|e| contract(e.to_string()))?
                        .computation
                        .clone()
                        .ok_or_else(|| contract("shooting completion absent"))?,
                )
                .await
                .map_err(relation)?;
            header.finish().await.map_err(relation)?;
        }
        let trace = report
            .and_then(|r| r.strategy.as_ref())
            .or_else(|| match &self.report {
                Err(error) => error.strategy_trace(),
                _ => None,
            });
        use pse_relations::generated::runtime::solve_strategy_events;
        let mut strategy_events =
            crate::workflow::result_export::Rows::<solve_strategy_events::Row>::new(
                request,
                registry,
                &pool,
                &cancel,
                &validation,
            )
            .map_err(relation)?;
        if let Some(trace) = trace
            && request.wants(solve_strategy_events::RELATION_ID)
        {
            strategy_events
                .strategy_events(trace, self.run_id, 0)
                .await?;
        }
        strategy_events.finish().await.map_err(relation)?;
        use pse_relations::generated::runtime::solve_strategy_products;
        let mut products =
            crate::workflow::result_export::Rows::<solve_strategy_products::Row>::new(
                request,
                registry,
                &pool,
                &cancel,
                &validation,
            )
            .map_err(relation)?;
        if let Some(trace) = trace
            && request.wants(solve_strategy_products::RELATION_ID)
        {
            products.strategy_products(trace, self.run_id, 0).await?;
        }
        products.finish().await.map_err(relation)?;
        Ok(())
    }
}
