// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Registry projections retain source-coordinate evidence and columnar resource ownership.
use super::*;
use pse_model::generated::identities::RunId;
use pse_relations::columnar::{Collection, FieldCheckedBatch, RelationRow};
type Tables = BTreeMap<SemanticId, FieldCheckedBatch>;

/// Publish retained admission without requiring a numerical attempt.
pub(super) fn admission_tables(
    runtime: &Runtime,
    decision: &pse_backend_native::routing::Decision,
    request_identity: pse_ids::ContentHash,
    step: i64,
) -> Result<Tables, WorkflowError> {
    use pse_model::HeapUsage;
    let route = decision.row(request_identity, step);
    let structure = decision
        .structure
        .as_ref()
        .map(|assessment| assessment.row(request_identity, step));
    let bytes = route
        .heap_bytes()
        .saturating_add(structure.as_ref().map_or(0, HeapUsage::heap_bytes))
        .saturating_add(4096);
    export(runtime, bytes, |columns| {
        columns.push(route).map_err(relation)?;
        if let Some(structure) = structure {
            columns.push(structure).map_err(relation)?;
        }
        Ok(())
    })
}
impl ModelingSolvePreparation {
    /// Checked route and original structural facts, keyed by the owning request and step.
    pub fn admission_tables(
        &self,
        request_identity: pse_ids::ContentHash,
        step: i64,
    ) -> Result<Tables, WorkflowError> {
        let decision = self
            .solve
            .route_decision()
            .ok_or_else(|| contract("algebraic preparation admission facts absent"))?;
        admission_tables(&self.source.runtime, decision, request_identity, step)
    }
}
impl ModelingDiagnosticPreparation {
    /// Checked retained refusal or admission facts; preparing these tables executes no native solver.
    pub fn admission_tables(
        &self,
        runtime: &Runtime,
        request_identity: pse_ids::ContentHash,
        step: i64,
    ) -> Result<Tables, WorkflowError> {
        admission_tables(runtime, &self.route_decision, request_identity, step)
    }
}
impl ModelingDiagnostics {
    /// The route and original witness assessed before numerical diagnostics.
    pub fn admission_tables(
        &self,
        request_identity: pse_ids::ContentHash,
        step: i64,
    ) -> Result<Tables, WorkflowError> {
        admission_tables(&self.runtime, &self.route_decision, request_identity, step)
    }
}

fn real_evidence(
    value: f64,
) -> (
    pse_model::generated::enums::ModelingRealValueKind,
    Option<f64>,
) {
    use pse_model::generated::enums::ModelingRealValueKind as K;
    if value.is_finite() {
        (K::Finite, Some(value))
    } else if value == f64::INFINITY {
        (K::PositiveInfinity, None)
    } else if value == f64::NEG_INFINITY {
        (K::NegativeInfinity, None)
    } else {
        (K::Indeterminate, None)
    }
}

/// Owned generated analysis table and the native attempts it describes.
#[derive(Debug)]
pub struct ModelingNativeAnalysis {
    /// Generated relation name identifies the analysis and its evidence scope.
    pub relation: &'static str,
    /// Checked source-coordinate table; clones retain their columnar owner.
    pub table: FieldCheckedBatch,
    /// Original native attempts, ordered as described by the relation.
    pub attempts: Vec<pse_backend_native::solve::SolveReport>,
    _owner: Arc<pse_columnar::AllocationLease>,
}

#[cfg(feature = "solver-highs")]
#[path = "native_analysis_tables.rs"]
mod native_analysis;

fn export(
    runtime: &Runtime,
    bytes: usize,
    build: impl FnOnce(&mut Collection<'_>) -> Result<(), WorkflowError>,
) -> Result<Tables, WorkflowError> {
    let _scratch = runtime.shared.math().reserve(
        "modeling:analysis-row-copy",
        bytes
            .checked_mul(2)
            .and_then(|n| n.checked_add(4096))
            .ok_or_else(|| contract("analysis row copy extent"))?,
    )?;
    let pool = runtime.shared.pool();
    let cancel = pse_columnar::CancellationToken::new();
    let validation = runtime.validation_context()?;
    let mut columns = Collection::new(&runtime.registry, &pool, &cancel, &validation);
    build(&mut columns)?;
    Ok(columns
        .finish()
        .map_err(relation)?
        .into_values()
        .map(|b| (b.relation_id(), b))
        .collect())
}
fn one<T: RelationRow>(
    runtime: &Runtime,
    bytes: usize,
    row: impl FnOnce() -> T,
) -> Result<FieldCheckedBatch, WorkflowError> {
    let mut tables = export(runtime, bytes, |columns| {
        columns.push(row()).map_err(relation)
    })?;
    tables
        .pop_first()
        .map(|(_, batch)| batch)
        .ok_or_else(|| contract("analysis table absent"))
}
/// Project one structured failure using the shared findings relation.
pub(in crate::workflow) fn finding_row(
    run_id: RunId,
    ordinal: i64,
    f: &pse_model::diagnostic::BoundaryDiagnostic,
) -> pse_model::generated::runtime::modeling_findings::Row {
    super::super::diagnostic_rows::project_finding(run_id, ordinal, f)
}

fn export_finding_rows(
    runtime: &Runtime,
    bytes: usize,
    rows: impl Iterator<Item = pse_model::generated::runtime::modeling_findings::Row>,
) -> Result<FieldCheckedBatch, WorkflowError> {
    use pse_model::generated::runtime::modeling_findings::Row;
    let mut tables = export(runtime, bytes, |columns| {
        columns.ensure::<Row>().map_err(relation)?;
        for row in rows {
            columns.push(row).map_err(relation)?;
        }
        Ok(())
    })?;
    tables
        .pop_first()
        .map(|(_, table)| table)
        .ok_or_else(|| contract("findings relation absent"))
}

/// Pre-copy admission for the selected route receipt, excluding unrelated structural payloads.
pub(in crate::workflow) fn route_copy(
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    decision: &pse_backend_native::routing::Decision,
) -> Result<pse_columnar::MemoryReservation, WorkflowError> {
    let entries = decision
        .eligibility
        .iter()
        .try_fold(0usize, |total, entry| {
            total
                .checked_add(entry.reasons.len())
                .and_then(|n| n.checked_add(entry.evidence.len()))
                .and_then(|n| n.checked_add(entry.artifacts.len()))
                .ok_or_else(|| contract("route receipt copy extent"))
        })?;
    crate::workflow::result_export::working(
        pool,
        "result:route-receipt-copy",
        &[
            (decision.classes.len(), 8),
            (decision.evidence.len(), 32),
            (decision.artifacts.len(), 32),
            (decision.eligibility.len(), 256),
            (entries, 32),
            (1, 4096),
        ],
    )
}
pub(in crate::workflow) fn structure_copy(
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    assessment: &pse_backend_native::structural::Assessment,
) -> Result<pse_columnar::MemoryReservation, WorkflowError> {
    crate::workflow::result_export::working(
        pool,
        "result:structural-receipt-copy",
        &[(assessment.retained_bytes(), 4)],
    )
}
impl ModelingDiagnostics {
    /// Diagnostics inventory is independent of requested payload validation.
    pub fn table_ids(&self) -> Vec<SemanticId> {
        use pse_relations::generated::runtime::*;
        vec![
            modeling_diagnostics::RELATION_ID,
            modeling_findings::RELATION_ID,
            route_decisions::RELATION_ID,
            structural_assessments::RELATION_ID,
        ]
    }
    /// Every advertised relation through its selected cursor.
    pub fn tables(&self) -> Result<Tables, WorkflowError> {
        crate::workflow::result_export::collect_tables(self.table_ids(), &self.runtime, |id| {
            self.cursor_by_id(id, 1024).map_err(Arc::new)
        })
        .map_err(WorkflowError::Shared)
    }
    fn table_by_id(&self, id: SemanticId) -> Result<FieldCheckedBatch, WorkflowError> {
        crate::workflow::result_export::collect(self.cursor_by_id(id, 1024)?, &self.runtime, id)
            .map_err(WorkflowError::Shared)
    }
    /// The selected relation; an unrelated intrinsic defect is not consulted.
    pub fn table(&self, name: &str) -> Result<FieldCheckedBatch, WorkflowError> {
        let id = self
            .runtime
            .registry
            .relation(name)
            .ok_or_else(|| contract("unknown diagnostic relation"))?
            .id;
        self.table_by_id(id)
    }
    /// Independent bounded projection of retained diagnostics.
    pub fn table_cursor(
        &self,
        name: &str,
        rows: usize,
    ) -> Result<crate::workflow::ResultCursor<'_>, WorkflowError> {
        let id = self
            .runtime
            .registry
            .relation(name)
            .ok_or_else(|| contract("unknown diagnostic relation"))?
            .id;
        self.cursor_by_id(id, rows)
    }
    /// Owning bounded selected transport for foreign-language streams.
    pub fn into_table_cursor(
        self: Arc<Self>,
        name: &str,
        rows: usize,
    ) -> Result<crate::workflow::ResultCursor<'static>, WorkflowError> {
        let runtime = &self.runtime;
        let id = runtime
            .registry
            .relation(name)
            .ok_or_else(|| contract("unknown result relation"))?
            .id;
        if !self.table_ids().contains(&id) {
            return Err(contract("relation is not part of result"));
        }
        if let Some(error) = self.encodings.failure(id) {
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
                    .map_err(|error| self.encodings.record(id, error))
            },
        )
    }
    fn cursor_by_id(
        &self,
        id: SemanticId,
        rows: usize,
    ) -> Result<crate::workflow::ResultCursor<'_>, WorkflowError> {
        if !self.table_ids().contains(&id) {
            return Err(contract("relation is not part of diagnostics"));
        }
        if let Some(error) = self.encodings.failure(id) {
            return Err(WorkflowError::Shared(error));
        }
        let spec = self
            .runtime
            .registry
            .relation_by_id(id)
            .ok_or_else(|| contract("diagnostic declaration absent"))?;
        let schema = pse_schema::arrow::relation_schema_ref(&self.runtime.registry, spec)
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
                    .map_err(|error| self.encodings.record(id, error))
            },
        )
    }
    async fn project(
        &self,
        request: &crate::workflow::result_export::Projection,
    ) -> Result<(), WorkflowError> {
        use pse_math::index::{GlobalCol, GlobalRow, TiSlice};
        use pse_model::generated::runtime::{modeling_diagnostics::*, modeling_findings::*};
        let (row_ids, column_ids): (
            &TiSlice<GlobalRow, SemanticId>,
            &TiSlice<GlobalCol, SemanticId>,
        ) = (
            self.rows.as_slice().as_ref(),
            self.columns.as_slice().as_ref(),
        );
        let pool = self.runtime.shared.pool();
        let cancel = request.cancel.clone();
        let validation = self.runtime.validation_context()?;
        let mut columns = crate::workflow::result_export::SelectedCollection::new(
            request,
            &self.runtime.registry,
            &pool,
            &cancel,
            &validation,
        );
        columns
            .ensure::<RuntimeModelingDiagnosticsRow>()
            .map_err(relation)?;
        columns
            .ensure::<RuntimeModelingFindingsRow>()
            .map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::route_decisions::Row>()
            .map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::structural_assessments::Row>()
            .map_err(relation)?;
        if request.wants(pse_relations::generated::runtime::route_decisions::RELATION_ID)
            && !columns.skip_next::<pse_model::generated::runtime::route_decisions::Row>()
        {
            let _copy = route_copy(&pool, &self.route_decision)?;
            columns
                .push(self.route_decision.row(self.admission_identity, 0))
                .await
                .map_err(relation)?;
        }
        if request.wants(pse_relations::generated::runtime::structural_assessments::RELATION_ID)
            && !columns.skip_next::<pse_model::generated::runtime::structural_assessments::Row>()
            && let Some(assessment) = &self.route_decision.structure
        {
            let _copy = structure_copy(&pool, assessment)?;
            columns
                .push(assessment.row(self.admission_identity, 0))
                .await
                .map_err(relation)?;
        }
        if request.wants(pse_relations::generated::runtime::modeling_diagnostics::RELATION_ID)
            && !columns.skip_next::<RuntimeModelingDiagnosticsRow>()
        {
            for (ids, nominals) in [
                (&self.rows, &self.row_nominals),
                (&self.columns, &self.variable_nominals),
            ] {
                if ids.len() != nominals.len()
                    || ids.iter().zip(nominals).any(|(id, (source, value))| {
                        id != source || !value.is_finite() || *value <= 0.
                    })
                {
                    return Err(contract(
                        "diagnostic nominal source coordinate extent or value",
                    ));
                }
            }
            if self.matrix.as_ref().is_some_and(|m| {
                m.row_norms.len() != self.rows.len()
                    || m.column_norms.len() != self.columns.len()
                    || m.parallel_rows.iter().any(|p| {
                        p.first.get() >= self.rows.len() || p.second.get() >= self.rows.len()
                    })
                    || m.parallel_columns.iter().any(|p| {
                        p.first.get() >= self.columns.len() || p.second.get() >= self.columns.len()
                    })
                    || m.modes.iter().any(|mode| {
                        (!mode.left.is_empty() && mode.left.len() != self.rows.len())
                            || (!mode.right.is_empty() && mode.right.len() != self.columns.len())
                    })
            }) {
                return Err(contract("diagnostic matrix source coordinate extent"));
            }
            let mode_scalars = self.matrix.as_ref().map_or(Ok(0usize), |m| {
                m.modes.iter().try_fold(0usize, |total, mode| {
                    total
                        .checked_add(mode.left.len())
                        .and_then(|n| n.checked_add(mode.right.len()))
                        .ok_or_else(|| contract("diagnostic mode copy extent"))
                })
            })?;
            let names = self.statistics.keys().try_fold(0usize, |n, name| {
                n.checked_add(name.len())
                    .ok_or_else(|| contract("diagnostic statistic copy extent"))
            })?;
            let _copy = crate::workflow::result_export::working(
                &pool,
                "result:diagnostics-row-copy",
                &[
                    (self.profile.len(), 1),
                    (self.point.scalars.len(), 32),
                    (self.rows.len(), 16),
                    (self.columns.len(), 16),
                    (self.row_nominals.len(), 32),
                    (self.variable_nominals.len(), 32),
                    (self.statistics.len(), 32),
                    (names, 1),
                    (mode_scalars, 32),
                    (self.matrix.as_ref().map_or(0, |m| m.row_norms.len()), 32),
                    (self.matrix.as_ref().map_or(0, |m| m.column_norms.len()), 32),
                    (
                        self.matrix.as_ref().map_or(0, |m| m.parallel_rows.len()),
                        48,
                    ),
                    (
                        self.matrix.as_ref().map_or(0, |m| m.parallel_columns.len()),
                        48,
                    ),
                    (self.matrix.as_ref().map_or(0, |m| m.modes.len()), 64),
                ],
            )?;
            let matrix = self
                .matrix
                .as_ref()
                .map(|m| RuntimeModelingDiagnosticsFieldMatrix {
                    rank: m.rank as i64,
                    cutoff: m.cutoff,
                    row_norms: self
                        .rows
                        .iter()
                        .zip(&m.row_norms)
                        .map(
                            |(id, v)| RuntimeModelingDiagnosticsFieldMatrixRowNormsItem {
                                source_id: *id,
                                value: *v,
                            },
                        )
                        .collect(),
                    column_norms: self
                        .columns
                        .iter()
                        .zip(&m.column_norms)
                        .map(
                            |(id, v)| RuntimeModelingDiagnosticsFieldMatrixColumnNormsItem {
                                source_id: *id,
                                value: *v,
                            },
                        )
                        .collect(),
                    parallel_rows: m
                        .parallel_rows
                        .iter()
                        .map(|p| RuntimeModelingDiagnosticsFieldMatrixParallelRowsItem {
                            first_id: row_ids[p.first],
                            second_id: row_ids[p.second],
                            cosine: p.cosine,
                        })
                        .collect(),
                    parallel_columns: m
                        .parallel_columns
                        .iter()
                        .map(
                            |p| RuntimeModelingDiagnosticsFieldMatrixParallelColumnsItem {
                                first_id: column_ids[p.first],
                                second_id: column_ids[p.second],
                                cosine: p.cosine,
                            },
                        )
                        .collect(),
                    modes: m
                        .modes
                        .iter()
                        .map(|mode| RuntimeModelingDiagnosticsFieldMatrixModesItem {
                            value: mode.value,
                            left: self
                                .rows
                                .iter()
                                .zip(&mode.left)
                                .map(|(id, v)| {
                                    RuntimeModelingDiagnosticsFieldMatrixModesItemLeftItem {
                                        source_id: *id,
                                        value: *v,
                                    }
                                })
                                .collect(),
                            right: self
                                .columns
                                .iter()
                                .zip(&mode.right)
                                .map(|(id, v)| {
                                    RuntimeModelingDiagnosticsFieldMatrixModesItemRightItem {
                                        source_id: *id,
                                        value: *v,
                                    }
                                })
                                .collect(),
                        })
                        .collect(),
                });
            columns
                .push(RuntimeModelingDiagnosticsRow {
                    run_id: self.run_id,
                    source_identity: self.source_identity,
                    numerical_identity: self.numerical_identity,
                    profile: self.profile.clone(),
                    complete: self.complete,
                    point: self
                        .point
                        .scalars
                        .iter()
                        .map(|(id, v)| {
                            let (kind, value) = real_evidence(*v);
                            RuntimeModelingDiagnosticsFieldPointItem {
                                source_id: *id,
                                kind,
                                value,
                            }
                        })
                        .collect(),
                    rows: self.rows.clone(),
                    columns: self.columns.clone(),
                    row_nominals: self
                        .row_nominals
                        .iter()
                        .map(
                            |(id, value)| RuntimeModelingDiagnosticsFieldRowNominalsItem {
                                source_id: *id,
                                value: *value,
                            },
                        )
                        .collect(),
                    variable_nominals: self
                        .variable_nominals
                        .iter()
                        .map(
                            |(id, value)| RuntimeModelingDiagnosticsFieldVariableNominalsItem {
                                source_id: *id,
                                value: *value,
                            },
                        )
                        .collect(),
                    statistics: self
                        .statistics
                        .iter()
                        .map(
                            |(name, count)| RuntimeModelingDiagnosticsFieldStatisticsItem {
                                name: name.clone(),
                                count: *count as i64,
                            },
                        )
                        .collect(),
                    matrix,
                })
                .await
                .map_err(relation)?;
        }
        columns
            .ensure::<RuntimeModelingFindingsRow>()
            .map_err(relation)?;
        if request.wants(pse_relations::generated::runtime::modeling_findings::RELATION_ID) {
            for (ordinal, f) in self.findings.iter().enumerate() {
                columns
                    .push(finding_row(self.run_id, ordinal as i64, f))
                    .await
                    .map_err(relation)?;
            }
        }
        columns.finish().await.map_err(relation)
    }
}
impl ModelingInitializationReport {
    /// The findings of the initialization run as a checked `modeling_findings` relation.
    pub fn findings_table(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        export_finding_rows(
            &self.runtime,
            self._owner.size(),
            self.failure
                .iter()
                .map(|failure| finding_row(self.run_id, 0, failure))
                .chain(
                    self.attempts
                        .iter()
                        .enumerate()
                        .filter_map(|(index, attempt)| {
                            attempt
                                .diagnostic()
                                .map(|failure| finding_row(self.run_id, index as i64 + 1, &failure))
                        }),
                ),
        )
    }
    /// The modeling initialization row as a checked `modeling_initializations` relation.
    pub fn table(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        use pse_model::generated::runtime::modeling_initializations::*;
        one(&self.runtime, self._owner.size(), || {
            RuntimeModelingInitializationsRow {
                run_id: self.run_id,
                complete: self.completed,
                failure: self.failure.as_ref().map(ToString::to_string),
                failure_ordinal: self.failure.as_ref().map(|_| 0),
                committed: self.committed.as_ref().map(|v| {
                    v.iter()
                        .map(
                            |(id, value)| RuntimeModelingInitializationsFieldCommittedItem {
                                source_id: *id,
                                value: *value,
                            },
                        )
                        .collect()
                }),
                discrete: self.discrete,
                discrete_assignment: self
                    .discrete_assignment
                    .iter()
                    .map(
                        |(id, value)| RuntimeModelingInitializationsFieldDiscreteAssignmentItem {
                            source_id: *id,
                            value: *value,
                        },
                    )
                    .collect(),
                attempts: self
                    .attempts
                    .iter()
                    .enumerate()
                    .map(|(index, a)| {
                        let kind = a.step.kind();
                        let (stage, fraction) = match &a.step {
                            ModelingInitializationStep::Stage(s) => (Some(s.clone()), None),
                            ModelingInitializationStep::Homotopy(f) => (None, Some(*f)),
                            ModelingInitializationStep::Original => (None, None),
                        };
                        RuntimeModelingInitializationsFieldAttemptsItem {
                            kind,
                            stage,
                            fraction,
                            result_id: a.result.as_ref().ok().map(|r| r.run_id),
                            accepted: a.accepted(),
                            error: a.result.as_ref().err().map(engines::bounded_error),
                            failure_ordinal: a.diagnostic().map(|_| index as i64 + 1),
                            interruption_class: a.interruption.as_ref().map(|i| i.class),
                            interruption: a.interruption.as_ref().map(engines::bounded_error),
                        }
                    })
                    .collect(),
            }
        })
    }
}

impl ModelingDiagnosticSamples {
    /// The findings of the diagnostic samples as a checked `modeling_findings` relation.
    pub fn findings_table(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        export_finding_rows(
            &self.runtime,
            self._owner.size(),
            self.outcomes
                .iter()
                .enumerate()
                .filter_map(|(index, (_, result))| {
                    result
                        .as_ref()
                        .err()
                        .map(|error| finding_row(self.run_id, index as i64, error))
                }),
        )
    }
    /// The modeling diagnostic samples row as a checked `modeling_diagnostic_samples` relation.
    pub fn table(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        use pse_model::generated::runtime::modeling_diagnostic_samples::*;
        one(&self.runtime, self._owner.size(), || {
            RuntimeModelingDiagnosticSamplesRow {
                run_id: self.run_id,
                unattempted: self.unattempted as i64,
                stop: self.stop,
                outcomes: self
                    .outcomes
                    .iter()
                    .enumerate()
                    .map(|(index, (id, result))| {
                        RuntimeModelingDiagnosticSamplesFieldOutcomesItem {
                            sample_id: *id,
                            report_id: result.as_ref().ok().map(|r| r.run_id),
                            error_class: result.as_ref().err().map(|e| e.class),
                            error: result.as_ref().err().map(engines::bounded_error),
                            failure_ordinal: result.as_ref().err().map(|_| index as i64),
                        }
                    })
                    .collect(),
            }
        })
    }
}
impl ModelingNonlinearExplanation {
    /// The findings of the nonlinear explanation as a checked `modeling_findings` relation.
    pub fn findings_table(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        export_finding_rows(
            &self.runtime,
            self._owner.size(),
            self.stop
                .iter()
                .map(|error| finding_row(self.run_id, 0, error))
                .chain(
                    self.attempts
                        .iter()
                        .enumerate()
                        .filter_map(|(index, attempt)| {
                            attempt
                                .diagnostic()
                                .map(|error| finding_row(self.run_id, index as i64 + 1, &error))
                        }),
                ),
        )
    }
    /// The modeling nonlinear explanation row as a checked `modeling_nonlinear_explanations` relation.
    pub fn table(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        use pse_model::generated::runtime::modeling_nonlinear_explanations::*;
        one(&self.runtime, self._owner.size(), || {
            RuntimeModelingNonlinearExplanationsRow {
                run_id: self.run_id,
                source_identity: self.source_identity,
                complete: self.complete,
                stop: self.stop.as_ref().map(engines::bounded_error),
                failure_ordinal: self.stop.as_ref().map(|_| 0),
                candidate_rows: self.candidate_rows.iter().copied().collect(),
                background_variables: self.background_variables.clone(),
                penalty_tolerance: self.penalty_tolerance,
                nominals: self
                    .nominals
                    .iter()
                    .map(
                        |(id, v)| RuntimeModelingNonlinearExplanationsFieldNominalsItem {
                            source_id: *id,
                            value: *v,
                        },
                    )
                    .collect(),
                attempts: self
                    .attempts
                    .iter()
                    .enumerate()
                    .map(
                        |(index, a)| RuntimeModelingNonlinearExplanationsFieldAttemptsItem {
                            omitted: a.omitted.iter().copied().collect(),
                            observation: a.observation,
                            penalty: a.penalty,
                            result_id: a.result.as_ref().ok().map(|r| r.run_id),
                            error: a.result.as_ref().err().map(engines::bounded_error),
                            failure_ordinal: a.diagnostic().map(|_| index as i64 + 1),
                            interruption_class: a.interruption.as_ref().map(|d| d.class),
                            interruption: a.interruption.as_ref().map(engines::bounded_error),
                        },
                    )
                    .collect(),
            }
        })
    }
}
