// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native coefficient diagnostics retain source identities and their LP/MIP scope.
use super::*;
use pse_backend_native::{CoefficientProblem, highs, quality::Tolerances, solve::*, transport};
use pse_model::generated::identities::RunId;

/// An independent diagnostic attempt; a relaxed candidate is never a model result.
#[derive(Debug)]
pub struct ModelingLinearDiagnostics {
    /// Identity of this diagnostic run.
    pub run_id: RunId,
    /// The diagnostic model's own solve.
    pub attempt: SolveReport,
    /// Native diagnostics of that solve, in original coordinates.
    pub evidence: highs::diagnostics::Report,
    pub(in crate::workflow::modeling) runtime: Runtime,
    pub(in crate::workflow::modeling) source_identity: pse_ids::ContentHash,
    pub(in crate::workflow::modeling) numerical_identity: pse_ids::ContentHash,
    pub(in crate::workflow::modeling) rows: Vec<SemanticId>,
    pub(in crate::workflow::modeling) columns: Vec<SemanticId>,
    pub(in crate::workflow::modeling) _owner: Arc<pse_columnar::AllocationLease>,
}
impl ModelingPackage {
    /// Use HiGHS IIS, rays, ranging and explicit-penalty relaxation on an admitted
    /// affine coefficient snapshot. Nonlinear models and undisclosed domains refuse.
    pub async fn diagnose_linear(
        &self,
        mut prepared: ModelingDiagnosticPreparation,
        request: highs::diagnostics::Request,
        controls: Controls,
        maximum_entries: usize,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingLinearDiagnostics, WorkflowError> {
        controls
            .validate()
            .map_err(crate::math::MathRuntimeError::from)?;
        let service = self.runtime.shared.math();
        // Coefficients belong to this diagnostic consumer; general diagnostic
        // preparation leaves optional class evidence unassessed.
        prepared.model.case = service
            .discover_class(
                prepared.model.case,
                prepared.model.values.clone(),
                &crate::math::solves::SolverProfile {
                    intent: SolveIntent::FeasiblePoint,
                    selection: SolverSelection::Explicit(Backend::Highs),
                    ..Default::default()
                },
            )
            .await?;
        let product = prepared.model.case.compiled();
        let coefficients = product.coefficients.clone().ok_or_else(|| contract("linear diagnostics require an admitted affine coefficient representation and discharged domains"))?;
        if coefficients.hessian.val().iter().any(|v| *v != 0.) {
            return Err(contract("linear diagnostics require an affine objective"));
        }
        if product.coefficient_values.iter().any(|(id, bits)| {
            prepared.model.values.scalars.get(id).map(|v| v.to_bits()) != Some(*bits)
        }) {
            return Err(contract(
                "diagnostic values differ from coefficient assumptions",
            ));
        }
        let plan = product.plan.clone();
        let source_identity = plan.structure().key();
        let numerical_identity = prepared.numerics.key;
        let columns = plan.columns().to_vec();
        let rows = plan
            .structure()
            .rows()
            .iter()
            .map(|r| r.id)
            .collect::<Vec<_>>();
        let extent = coefficients
            .constraints
            .val()
            .len()
            .checked_add(rows.len())
            .and_then(|n| n.checked_add(plan.columns().len()))
            .ok_or_else(|| contract("linear diagnostic extent"))?;
        if maximum_entries == 0 || extent > maximum_entries {
            return Err(contract("linear diagnostic entry budget"));
        }
        let normalization = pse_math::normalization::Normalization::from_policy(
            &prepared.numerics,
            plan.columns(),
            &rows,
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        let tolerances = Tolerances::from_policy(&prepared.numerics, plan.columns(), &rows)
            .map_err(crate::math::MathRuntimeError::from)?;
        let accuracy =
            ResolvedAccuracy::resolve(&prepared.numerics.policy, &tolerances, &normalization)
                .map_err(crate::math::MathRuntimeError::from)?;
        let allowance = extent
            .checked_mul(512)
            .and_then(|n| n.checked_add(controls.report_allowance().ok()?))
            .ok_or_else(|| contract("linear diagnostic storage extent"))?;
        let owner = service.reserve("modeling:linear-diagnostics", allowance)?;
        let retained = owner.clone();
        let assembly = service.assemble(prepared.model.case).await?;
        let (attempt, evidence, owner) = service
            .with_worker(assembly, prepared.providers, cancel, move |worker| {
                let original = CoefficientProblem::from_plan(&plan, coefficients.as_ref().clone())?;
                let (problem, _) = transport::coefficients(&original, &normalization, None)?;
                let request = transport::diagnostic_request(&request, &normalization)?;
                let execution = Execution::new(worker.cancellation().clone(), &controls);
                let compatibility = Compatibility {
                    layout: problem.contract.identity,
                    profile: accuracy.key()?,
                    data: problem.assumptions,
                    backend: Backend::Highs,
                };
                let mut session = highs::Session::new(&problem, None, compatibility)?;
                let mut attempt = session.solve(
                    &problem,
                    &normalization,
                    &controls,
                    &accuracy,
                    &highs::Settings::default(),
                    execution.clone(),
                    &tolerances.normalized(&normalization)?,
                    None,
                )?;
                let mut evidence = session.diagnose(&problem, &request, &execution)?;
                transport::recover(&mut attempt, &normalization, &original.contract)?;
                transport::recover_diagnostics(
                    &mut evidence,
                    &normalization,
                    &coefficients.row_constants,
                    &original.contract,
                )?;
                Ok((attempt, evidence, retained))
            })
            .await?;
        Ok(ModelingLinearDiagnostics {
            run_id: pse_operations::mint_id(),
            runtime: self.runtime.clone(),
            source_identity,
            numerical_identity,
            rows,
            columns,
            attempt,
            evidence,
            _owner: owner,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn kernel_linear_diagnostics_use_native_iis_and_refuse_nonlinear_models() {
        let rt = super::super::super::super::tests::runtime();
        let physical = super::super::super::super::tests::physical();
        for (rhs, linear) in [("x", true), ("x*x", false)] {
            let source = format!(
                "package p {{ def Root {{ var x:Scalar; eq lo:{rhs}>=2; eq hi:{rhs}<=1; annotation start x(0); }} }}"
            );
            let declarations = pse_authoring::language::parse(
                &source,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = declarations
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = rt.modeling_package(declarations, physical.clone()).unwrap();
            let cancel = crate::CancelSource::new();
            let analysis = ModelingAnalysis {
                root,
                instance: pse_modeling::specialize::root_instance(root),
                bindings: Bindings::default(),
                limits: Limits::default(),
                case: Default::default(),
                order: pse_kernels::DerivativeOrder::First,
                compiler: super::super::super::super::tests::compiler_profile(),
                solver: super::super::super::super::tests::profile(),
                numerical: Default::default(),
            };
            let prepared = package
                .prepare_diagnostics(&analysis, &cancel)
                .await
                .unwrap();
            let original_rows = prepared
                .model
                .case
                .compiled()
                .plan
                .structure()
                .rows()
                .iter()
                .map(|r| r.id)
                .collect::<std::collections::BTreeSet<_>>();
            let result = package
                .diagnose_linear(
                    prepared,
                    highs::diagnostics::Request {
                        iis: true,
                        ..Default::default()
                    },
                    Controls::default(),
                    1000,
                    &cancel,
                )
                .await;
            if linear {
                let result = result.unwrap();
                assert_eq!(result.attempt.termination.category, Termination::Infeasible);
                let iis = result.evidence.iis.as_ref().unwrap_or_else(|| {
                    panic!("native IIS unavailable: {:?}", result.evidence.unavailable)
                });
                assert!(!iis.relaxation_only);
                assert_eq!(
                    iis.rows
                        .iter()
                        .map(|(id, _)| *id)
                        .collect::<std::collections::BTreeSet<_>>(),
                    original_rows
                );
                let exported = result.into_export().unwrap();
                assert_eq!(exported.table.batch().num_rows(), 1);
                assert_eq!(exported.attempts.len(), 1);
            } else {
                assert!(
                    result
                        .err()
                        .unwrap()
                        .to_string()
                        .contains("affine coefficient")
                );
            }
        }
    }
}
