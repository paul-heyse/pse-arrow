// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One source-coordinate projection of native evidence, shared by Rust and Python.
use super::*;
use pse_model::generated::runtime::{
    modeling_jacobian_optimization as jacobian, modeling_linear_diagnostics as linear,
};

impl ModelingLinearDiagnostics {
    /// Transfer the diagnostic history into its generated owned transport.
    pub fn into_export(self) -> Result<ModelingNativeAnalysis, WorkflowError> {
        use linear::*;
        let d = &self.evidence;
        if d.primal_ray
            .as_ref()
            .is_some_and(|v| v.len() != self.columns.len())
            || d.dual_ray
                .as_ref()
                .is_some_and(|v| v.len() != self.rows.len())
            || d.iis.as_ref().is_some_and(|v| {
                v.column_status.len() != self.columns.len() || v.row_status.len() != self.rows.len()
            })
            || d.relaxation
                .as_ref()
                .and_then(|v| v.primal.as_ref())
                .is_some_and(|v| v.len() != self.columns.len())
            || d.ranging.values().any(|r| {
                [
                    r.value.len(),
                    r.objective.len(),
                    r.entering.len(),
                    r.leaving.len(),
                ]
                .iter()
                .any(|n| *n != r.ids.len())
            })
        {
            return Err(contract("linear diagnostic source coordinate extent"));
        }
        let endpoint =
            |index: i32| -> Result<(Option<SemanticId>, bool, Option<i64>), WorkflowError> {
                if index < 0 {
                    return Ok((None, false, Some(i64::from(index))));
                }
                let i = index as usize;
                if i < self.columns.len() {
                    return Ok((Some(self.columns[i]), false, None));
                }
                self.rows
                    .get(i - self.columns.len())
                    .copied()
                    .map(|id| (Some(id), true, None))
                    .ok_or_else(|| contract("native ranging augmented coordinate extent"))
            };
        for r in d.ranging.values() {
            for &i in r.entering.iter().chain(&r.leaving) {
                endpoint(i)?;
            }
        }
        let table = one(&self.runtime, self._owner.size(), || {
            RuntimeModelingLinearDiagnosticsRow {
                run_id: self.run_id,
                source_identity: self.source_identity,
                numerical_identity: self.numerical_identity,
                rows: self.rows.clone(),
                columns: self.columns.clone(),
                attempt: RuntimeModelingLinearDiagnosticsFieldAttempt {
                    termination: RuntimeModelingLinearDiagnosticsFieldAttemptTermination {
                        category: self.attempt.termination.category,
                        code: self.attempt.termination.code,
                        name: self.attempt.termination.name.clone(),
                    },
                    qualification: self.attempt.qualification,
                    validation_error: self.attempt.validation_failure().map(ToString::to_string),
                },
                primal_ray: d.primal_ray.as_ref().map(|v| {
                    self.columns
                        .iter()
                        .zip(v)
                        .map(
                            |(id, v)| RuntimeModelingLinearDiagnosticsFieldPrimalRayItem {
                                source_id: *id,
                                value: *v,
                            },
                        )
                        .collect()
                }),
                dual_ray: d.dual_ray.as_ref().map(|v| {
                    self.rows
                        .iter()
                        .zip(v)
                        .map(|(id, v)| RuntimeModelingLinearDiagnosticsFieldDualRayItem {
                            source_id: *id,
                            value: *v,
                        })
                        .collect()
                }),
                iis: d
                    .iis
                    .as_ref()
                    .map(|v| RuntimeModelingLinearDiagnosticsFieldIis {
                        columns: v
                            .columns
                            .iter()
                            .map(
                                |(id, c)| RuntimeModelingLinearDiagnosticsFieldIisColumnsItem {
                                    source_id: *id,
                                    code: i64::from(*c),
                                },
                            )
                            .collect(),
                        rows: v
                            .rows
                            .iter()
                            .map(|(id, c)| RuntimeModelingLinearDiagnosticsFieldIisRowsItem {
                                source_id: *id,
                                code: i64::from(*c),
                            })
                            .collect(),
                        column_status: self
                            .columns
                            .iter()
                            .zip(&v.column_status)
                            .map(|(id, c)| {
                                RuntimeModelingLinearDiagnosticsFieldIisColumnStatusItem {
                                    source_id: *id,
                                    code: i64::from(*c),
                                }
                            })
                            .collect(),
                        row_status: self
                            .rows
                            .iter()
                            .zip(&v.row_status)
                            .map(
                                |(id, c)| RuntimeModelingLinearDiagnosticsFieldIisRowStatusItem {
                                    source_id: *id,
                                    code: i64::from(*c),
                                },
                            )
                            .collect(),
                        relaxation_only: v.relaxation_only,
                    }),
                ranging: d
                    .ranging
                    .iter()
                    .flat_map(|(family, r)| {
                        r.ids.iter().enumerate().map(|(i, id)| {
                            // All augmented coordinates were checked above; this projection is total.
                            let project = |index: i32| {
                                let sentinel = (index < 0).then_some(i64::from(index));
                                let ix = usize::try_from(index).ok();
                                let row_slack = ix.is_some_and(|i| i >= self.columns.len());
                                let source_id = ix
                                    .and_then(|i| {
                                        if row_slack {
                                            self.rows.get(i - self.columns.len())
                                        } else {
                                            self.columns.get(i)
                                        }
                                    })
                                    .copied();
                                (source_id, row_slack, sentinel)
                            };
                            let (source_id, row_slack, sentinel) = project(r.entering[i]);
                            let entering =
                                RuntimeModelingLinearDiagnosticsFieldRangingItemEntering {
                                    source_id,
                                    row_slack,
                                    sentinel,
                                };
                            let (source_id, row_slack, sentinel) = project(r.leaving[i]);
                            let leaving = RuntimeModelingLinearDiagnosticsFieldRangingItemLeaving {
                                source_id,
                                row_slack,
                                sentinel,
                            };
                            RuntimeModelingLinearDiagnosticsFieldRangingItem {
                                family: family.clone(),
                                source_id: *id,
                                value: {
                                    let (kind, value) = real_evidence(r.value[i]);
                                    RuntimeModelingLinearDiagnosticsFieldRangingItemValue {
                                        kind,
                                        value,
                                    }
                                },
                                objective: {
                                    let (kind, value) = real_evidence(r.objective[i]);
                                    RuntimeModelingLinearDiagnosticsFieldRangingItemObjective {
                                        kind,
                                        value,
                                    }
                                },
                                entering,
                                leaving,
                            }
                        })
                    })
                    .collect(),
                relaxation: d.relaxation.as_ref().map(|r| {
                    RuntimeModelingLinearDiagnosticsFieldRelaxation {
                        operation_status: i64::from(r.operation_status),
                        restored_status:
                            RuntimeModelingLinearDiagnosticsFieldRelaxationRestoredStatus {
                                category: r.restored_status.category,
                                code: r.restored_status.code,
                                name: r.restored_status.name.clone(),
                            },
                        penalty: r.penalty,
                        primal: r.primal.as_ref().map(|v| {
                            self.columns
                                .iter()
                                .zip(v)
                                .map(|(id, v)| {
                                    RuntimeModelingLinearDiagnosticsFieldRelaxationPrimalItem {
                                        source_id: *id,
                                        value: *v,
                                    }
                                })
                                .collect()
                        }),
                    }
                }),
                unavailable: d
                    .unavailable
                    .iter()
                    .map(|(analysis, reason)| {
                        RuntimeModelingLinearDiagnosticsFieldUnavailableItem {
                            analysis: analysis.clone(),
                            reason: reason.clone(),
                        }
                    })
                    .collect(),
            }
        })?;
        Ok(ModelingNativeAnalysis {
            relation: "runtime.modeling_linear_diagnostics",
            table,
            attempts: vec![self.attempt],
            _owner: self._owner,
        })
    }
}

impl ModelingJacobianOptimization {
    /// Export scaled-matrix certificates alongside the physical point and original native stops.
    pub fn into_export(self) -> Result<ModelingNativeAnalysis, WorkflowError> {
        use jacobian::*;
        let table = one(&self.runtime, self._owner.size(), || {
            RuntimeModelingJacobianOptimizationRow{
            run_id:self.run_id,source_identity:self.source_identity,numerical_identity:self.numerical_identity,
            point:self.point.scalars.iter().map(|(id,v)|RuntimeModelingJacobianOptimizationFieldPointItem{source_id:*id,value:*v}).collect(),
            row_nominals:self.row_nominals.iter().map(|(id,v)|RuntimeModelingJacobianOptimizationFieldRowNominalsItem{source_id:*id,value:*v}).collect(),
            variable_nominals:self.variable_nominals.iter().map(|(id,v)|RuntimeModelingJacobianOptimizationFieldVariableNominalsItem{source_id:*id,value:*v}).collect(),
            tolerance:self.policy.tolerance,rank_relative:self.policy.rank_relative,multiplier_bound:self.policy.multiplier_bound,complete:self.evidence.complete,
            conditioning:self.evidence.conditioning.iter().map(|c|RuntimeModelingJacobianOptimizationFieldConditioningItem{
                weights:c.weights.iter().map(|(id,v)|RuntimeModelingJacobianOptimizationFieldConditioningItemWeightsItem{source_id:*id,value:*v}).collect(),residual_maximum:c.residual_maximum,pivot:c.pivot,
            }).collect(),
            degenerate:self.evidence.degenerate.iter().map(|d|RuntimeModelingJacobianOptimizationFieldDegenerateItem{
                rows:d.rows.clone(),irreducible_at_tolerance:d.irreducible_at_tolerance,
                certificate:RuntimeModelingJacobianOptimizationFieldDegenerateItemCertificate{
                    weights:d.certificate.weights.iter().map(|(id,v)|RuntimeModelingJacobianOptimizationFieldDegenerateItemCertificateWeightsItem{source_id:*id,value:*v}).collect(),residual_maximum:d.certificate.residual_maximum,pivot:d.certificate.pivot,
                },
            }).collect(),
            attempts:self.evidence.attempts.iter().map(|a|RuntimeModelingJacobianOptimizationFieldAttemptsItem{
                termination:RuntimeModelingJacobianOptimizationFieldAttemptsItemTermination{category:a.termination.category,code:a.termination.code,name:a.termination.name.clone()},qualification:a.qualification,validation_error:a.validation_failure().map(ToString::to_string),
            }).collect(),unavailable:self.evidence.unavailable.clone(),
        }
        })?;
        Ok(ModelingNativeAnalysis {
            relation: "runtime.modeling_jacobian_optimization",
            table,
            attempts: self.evidence.attempts,
            _owner: self._owner,
        })
    }
}
