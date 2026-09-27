// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepared original-model checks gate native sequence reuse without nested scheduling.
use super::*;
use super::results::{AssessmentScope, assessment_units, assess_observations};
use crate::math::{ExecutableCase, solves::{Outcome, SequenceAssessment}};
use pse_math::binding::CaseValues;
use std::sync::{Arc, Mutex, atomic::AtomicBool};

#[derive(Debug)]
pub(in crate::workflow) struct AssessedPoint {
    pub values: CaseValues,
    pub checks: Vec<ModelingCheck>,
    pub reports: Vec<ModelingReport>,
    pub error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    pub owner: Arc<pse_columnar::AllocationLease>,
}
#[derive(Debug)]
struct Program {
    prepared: ModelingSolvePreparation,
    executable: Option<Arc<ExecutableCase>>,
    scope: AssessmentScope,
    rows: Vec<SemanticId>,
    owner: Arc<pse_columnar::AllocationLease>,
}
pub(in crate::workflow) type Assessments = Arc<Mutex<Vec<Option<AssessedPoint>>>>;
#[derive(Debug)]
struct Assessor {
    run_id: SemanticId,
    programs: Vec<Program>,
    points: Assessments,
}
impl SequenceAssessment for Assessor {
    fn accepted(&mut self, attempt: usize, outcome: &Outcome, flag: &Arc<AtomicBool>) -> bool {
        let program = &self.programs[attempt];
        let prepared = &program.prepared;
        let mut point = AssessedPoint {
            values: prepared.model.values.clone(),
            checks: vec![],
            reports: vec![],
            error: None,
            owner: program.owner.clone(),
        };
        let evaluate =
            || -> Result<(CaseValues, Vec<ModelingCheck>, Vec<ModelingReport>), WorkflowError> {
                let mut values = prepared.model.values.clone();
                match outcome {
                    Outcome::Constant(_) => {}
                    Outcome::Native(native) => {
                        let candidate = native
                            .candidate
                            .as_ref()
                            .ok_or_else(|| contract("sequence attempt has no candidate"))?;
                        if candidate.primal.len() != native.variables.len() {
                            return Err(contract("sequence candidate coordinate extent"));
                        }
                        values.scalars.extend(
                            native
                                .variables
                                .iter()
                                .copied()
                                .zip(candidate.primal.iter().copied()),
                        );
                    }
                    Outcome::Rejected(error) => {
                        return Err(WorkflowError::Math(crate::math::MathRuntimeError::Shared(
                            error.clone(),
                        )));
                    }
                }
                let observed = if let Some(executable) = &program.executable {
                    let providers = prepared
                        .providers
                        .iter()
                        .map(|(key, p)| p.worker().map(|w| (*key, w)))
                        .collect::<Result<_, _>>()
                        .map_err(|e| {
                            crate::math::MathRuntimeError::from(
                                pse_backend_native::ProblemError::Provider(e),
                            )
                        })?;
                    let mut worker = executable.assembly.worker(providers, flag.clone());
                    let values = worker
                        .constraints(&values)
                        .map_err(crate::math::MathRuntimeError::from)?;
                    program.rows.iter().copied().zip(values).collect()
                } else {
                    BTreeMap::new()
                };
                let (mut checks, mut reports) = assess_observations(
                    self.run_id,
                    prepared.model.model.compiled(),
                    &values,
                    &observed,
                    prepared.solve.numerics(),
                    &prepared.source.quantities,
                    true,
                    Some(&program.scope),
                )?;
                for row in &mut checks {
                    row.step = attempt as i64;
                }
                for row in &mut reports {
                    row.step = attempt as i64;
                }
                Ok((values, checks, reports))
            };
        match evaluate() {
            Ok((values, checks, reports)) => {
                point.values = values;
                point.checks = checks;
                point.reports = reports;
            }
            Err(error) => point.error = Some(error.boundary_diagnostic()),
        }
        let accepted = point.error.is_none() && point.checks.iter().all(|c| c.satisfied);
        let Ok(mut points) = self.points.lock() else {
            return false;
        };
        points[attempt] = Some(point);
        accepted
    }
}
/// Prepare all original observations before the native sequence acquires its worker permit.
pub(in crate::workflow) async fn prepare(
    run_id: SemanticId,
    steps: &[ModelingSolvePreparation],
    cancel: &crate::CancelSource,
) -> Result<(Box<dyn SequenceAssessment>, Assessments), WorkflowError> {
    let mut programs = Vec::new();
    let points = Arc::new(Mutex::new(
        (0..steps.len()).map(|_| None).collect::<Vec<_>>(),
    ));
    for prepared in steps {
        let source = &prepared.source;
        let units = assessment_units(prepared.model.model.compiled());
        let scope = units.keys().copied().collect();
        let rows = units
            .into_values()
            .flatten()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let executable = if rows.is_empty() {
            None
        } else {
            Some(
                source
                    .runtime
                    .native()
                    .prepare_modeling_functions(
                        source.workspace.clone(),
                        prepared.model.model.clone(),
                        rows,
                        vec![],
                        pse_kernels::DerivativeOrder::Value,
                        prepared.compiler,
                        cancel,
                    )
                    .await?,
            )
        };
        let rows = executable
            .as_ref()
            .map(|p| p.assembly.structure().rows().iter().map(|r| r.id).collect())
            .unwrap_or_default();
        let owner = source.runtime.native().reserve(
            "modeling:sequence-assessment",
            results::result_bytes(prepared)?,
        )?;
        programs.push(Program {
            prepared: prepared.clone(),
            executable,
            scope,
            rows,
            owner,
        });
    }
    Ok((
        Box::new(Assessor {
            run_id,
            programs,
            points: points.clone(),
        }),
        points,
    ))
}
