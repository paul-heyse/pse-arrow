// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Named diagnostic evidence over the same model, source values and derivative programs.
#[cfg(feature = "solver-highs")]
#[path = "diagnostic_linear.rs"]
mod linear;
use super::*;
use crate::math::modeling::ModelingCasePreparation;
#[cfg(feature = "solver-highs")]
pub use linear::ModelingLinearDiagnostics;
use pse_model::generated::identities::RunId;
#[path = "diagnostic_nonlinear.rs"]
mod nonlinear;
#[path = "diagnostic_samples.rs"]
mod samples;
pub use nonlinear::{
    ElasticObservation, ModelingElasticAttempt, ModelingInfeasibilityCertificate,
    ModelingNonlinearExplanation, ModelingNonlinearPolicy,
};
use pse_math::{
    binding::CaseValues,
    diagnostics::{MatrixPolicy, MatrixReport, TermPolicy},
    index::{GlobalCol, GlobalRow, TiSlice},
};
use pse_model::{
    diagnostic::{BoundaryClass, BoundaryDiagnostic, Observation, Severity, SourceLocation},
    generated::enums::NumericalTarget,
    numerics::ResolvedNumericalPolicy,
};
pub use samples::{DiagnosticSampleStop, ModelingDiagnosticSamples};
use std::sync::Arc;

/// Diagnostic preparation resolves the same case and native dependencies without solver admission.
#[derive(Clone, Debug)]
pub struct ModelingDiagnosticPreparation {
    /// The prepared case, with its structure and start values.
    pub model: ModelingCasePreparation,
    /// Resolved physical nominals and numerical requirements of the case.
    pub numerics: Arc<ResolvedNumericalPolicy>,
    /// Retained original route and structural assessment, including admission refusals.
    pub route_decision: pse_backend_native::routing::Decision,
    /// Bound request identity, independent of a native numerical attempt.
    pub admission_identity: pse_ids::ContentHash,
    providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
}
impl ModelingDiagnosticPreparation {
    /// Candidate values may supply missing free coordinates, but cannot alter the
    /// fixed/parameter assumptions frozen into this prepared structure.
    fn validate_point(&self, values: &CaseValues) -> Result<(), WorkflowError> {
        let plan = &self.model.case.compiled().plan;
        let declared = plan
            .structure()
            .variables()
            .iter()
            .map(|v| v.port.id)
            .chain(plan.structure().parameters().iter().map(|p| p.id))
            .collect::<std::collections::BTreeSet<_>>();
        if values.scalars.keys().any(|id| !declared.contains(id))
            || self
                .model
                .case
                .compiled()
                .coefficient_values
                .iter()
                .any(|(id, bits)| values.scalars.get(id).map(|v| v.to_bits()) != Some(*bits))
        {
            return Err(contract(
                "diagnostic point changes frozen inputs or declared coordinates; prepare another case",
            ));
        }
        Ok(())
    }
}

/// Explicit thresholds and budgets; the caller selects the named knowledge profile.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelingDiagnosticPolicy {
    /// Name of the knowledge profile these thresholds come from.
    pub profile: String,
    /// Finding budget; further findings leave the report incomplete.
    pub maximum_findings: usize,
    /// Absolute part of the near-bound distance.
    pub near_bound_absolute: f64,
    /// Relative part of the near-bound distance, times the larger of the bound and nominal.
    pub near_bound_relative: f64,
    /// Bound violation above which a value is outside its bound.
    pub bound_violation: f64,
    /// Row residual, relative to the row nominal, above which it is large.
    pub residual: f64,
    /// Normalized magnitude below which a nonzero value is small.
    pub variable_small: f64,
    /// Normalized magnitude above which a value is large.
    pub variable_large: f64,
    /// Normalized magnitude at or below which a value counts as zero.
    pub variable_zero: f64,
    /// Normalized Jacobian magnitude below which a nonzero entry, row or column is small.
    pub jacobian_small: f64,
    /// Normalized Jacobian magnitude above which an entry, row or column is large.
    pub jacobian_large: f64,
    /// Dense matrix analysis budgets and tolerances.
    pub matrix: MatrixPolicy,
    /// Equation term analysis budgets and tolerances.
    pub terms: TermPolicy,
}
/// Named numerical evidence at one point of one prepared case.
#[derive(Clone, Debug)]
pub struct ModelingDiagnostics {
    /// Identity of this diagnostic run.
    pub run_id: RunId,
    pub(super) runtime: Runtime,
    pub(super) source_identity: pse_ids::ContentHash,
    pub(super) numerical_identity: pse_ids::ContentHash,
    pub(super) point: CaseValues,
    /// The policy's knowledge profile.
    pub profile: String,
    /// Findings in the order they were made.
    pub findings: Vec<BoundaryDiagnostic>,
    /// Structure counts of the case.
    pub statistics: BTreeMap<String, usize>,
    /// The retained original structural and capability assessment.
    pub route_decision: pse_backend_native::routing::Decision,
    /// Bound request identity, independent of a native numerical attempt.
    pub admission_identity: pse_ids::ContentHash,
    /// Dense analysis of the normalized Jacobian, when its budget allowed one; its rows and
    /// columns index [`Self::rows`] and [`Self::columns`].
    pub matrix: Option<MatrixReport<GlobalRow, GlobalCol>>,
    /// Every analysis ran within its budgets.
    pub complete: bool,
    /// Jacobian rows, as equation ids.
    pub rows: Vec<SemanticId>,
    /// Jacobian columns, as variable ids.
    pub columns: Vec<SemanticId>,
    /// Canonical physical row magnitudes used to normalize the reported Jacobian.
    pub row_nominals: Vec<(SemanticId, f64)>,
    /// Canonical physical variable magnitudes used to normalize its columns.
    pub variable_nominals: Vec<(SemanticId, f64)>,
    pub(in crate::workflow::modeling) _owner: Arc<pse_columnar::AllocationLease>,
}
impl ModelingDiagnosticPolicy {
    /// Finite, ordered thresholds and valid analysis budgets.
    ///
    /// # Errors
    /// A threshold or budget outside its range.
    pub fn validate(&self) -> Result<(), WorkflowError> {
        self.matrix
            .validate()
            .map_err(crate::math::MathRuntimeError::from)?;
        self.terms
            .validate()
            .map_err(crate::math::MathRuntimeError::from)?;
        if self.profile.is_empty()
            || self.maximum_findings == 0
            || self.maximum_findings > 100_000
            || [
                self.near_bound_absolute,
                self.near_bound_relative,
                self.bound_violation,
                self.residual,
                self.variable_small,
                self.variable_large,
                self.variable_zero,
                self.jacobian_small,
                self.jacobian_large,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.)
            || self.variable_small >= self.variable_large
            || self.jacobian_small >= self.jacobian_large
        {
            return Err(contract("invalid bounded diagnostics policy"));
        }
        Ok(())
    }
}
/// Class and severity of one declared numerical-diagnostics rule (ADR-0106, DP-21).
/// Structural and missing-value defects are invalid-model errors; scaling, conditioning
/// and near-bound observations are numerical warnings or information, never an invalid
/// model. Every rule this module declares has an explicit arm.
fn disposition(kind: &str) -> (BoundaryClass, Severity) {
    use BoundaryClass as C;
    use Severity as S;
    match kind {
        "structural.underdetermined" | "structural.overdetermined" | "variable.missing_value" => {
            (C::InvalidModel, S::Error)
        }
        "variable.nonfinite" => (C::Nonfinite, S::Error),
        "variable.unused"
        | "variable.only_in_inequalities"
        | "domain.potential_evaluation_error" => (C::InvalidModel, S::Info),
        "variable.outside_lower_bound"
        | "variable.outside_upper_bound"
        | "equation.large_residual"
        | "jacobian.parallel_rows"
        | "jacobian.parallel_columns"
        | "jacobian.numerical_rank_deficiency" => (C::Numerical, S::Warning),
        "variable.near_lower_bound"
        | "variable.near_upper_bound"
        | "variable.fixed_zero"
        | "variable.large_value"
        | "variable.small_value"
        | "jacobian.extreme_entry"
        | "jacobian.extreme_row"
        | "jacobian.extreme_column"
        | "jacobian.condition_estimate"
        | "equation.mismatched_term"
        | "equation.canceling_terms" => (C::Numerical, S::Info),
        "equation.term_evaluation_failed" => (C::TrialRejected, S::Warning),
        _ => (C::Internal, S::Error),
    }
}
fn finding(
    kind: &str,
    ids: impl IntoIterator<Item = SemanticId>,
    score: Option<f64>,
    threshold: Option<f64>,
) -> BoundaryDiagnostic {
    let (class, severity) = disposition(kind);
    let mut f =
        BoundaryDiagnostic::new(class, "modeling.diagnostics", ids, kind).with_severity(severity);
    if let Some(v) = score {
        f.observations.insert("value".into(), Observation::Real(v));
    }
    if let Some(v) = threshold {
        f.observations
            .insert("threshold".into(), Observation::Real(v));
    }
    f
}
impl ModelingDiagnostics {
    fn push(
        &mut self,
        mut diagnostic: BoundaryDiagnostic,
        maximum: usize,
        model: &pse_compiler::workspace::PreparedModeling,
    ) {
        if self.findings.len() >= maximum {
            self.complete = false;
            return;
        }
        attribute(&mut diagnostic, model);
        self.findings.push(diagnostic);
    }
}
/// Attach only authored lineage supplied by the selected compiler model.
pub(super) fn attribute(
    diagnostic: &mut BoundaryDiagnostic,
    model: &pse_compiler::workspace::PreparedModeling,
) {
    for id in &diagnostic.sources {
        let lineage = model.model.symbols.get(id).map(|s| &s.lineage).or_else(|| {
            model
                .model
                .equations
                .iter()
                .find(|r| r.id == *id)
                .map(|r| &r.lineage)
        });
        if let Some(l) = lineage {
            diagnostic.locations.push(SourceLocation {
                source: *id,
                path: l.path.clone(),
                name: None,
                start: None,
                end: None,
            });
        }
    }
}
impl ModelingPackage {
    /// Resolve starts, specifications, source precedence and nested providers without requesting a solve.
    pub async fn prepare_diagnostics(
        &self,
        analysis: &ModelingAnalysis,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingDiagnosticPreparation, WorkflowError> {
        let resolved = self
            .resolve_case(
                analysis.root,
                analysis.instance,
                analysis.bindings.clone(),
                analysis.limits,
                analysis.case.clone(),
                pse_kernels::DerivativeOrder::First,
                analysis.compiler,
                analysis.solver.clone(),
                analysis.numerical.clone(),
                Default::default(),
                true,
                cancel,
            )
            .await?;
        let route_decision = resolved.route_decision()?;
        let admission_identity = resolved.admission_identity(&route_decision)?;
        let physical = &resolved.model.case.compiled().quantities;
        let mut targets = resolved
            .model
            .case
            .compiled()
            .plan
            .numerical_targets(physical)
            .map_err(crate::math::MathRuntimeError::from)?;
        targets.extend(resolved.numerical.targets);
        let numerics = Arc::new(
            pse_math::numerics::resolve(
                physical,
                &targets,
                &resolved.numerical.declarations,
                &resolved.solver.numerics,
            )
            .map_err(crate::math::MathRuntimeError::from)?,
        );
        Ok(ModelingDiagnosticPreparation {
            model: resolved.model,
            numerics,
            route_decision,
            admission_identity,
            providers: resolved.providers,
        })
    }
    /// Diagnostic-only model analysis. It neither changes a candidate nor upgrades solver status.
    pub async fn diagnose_case(
        &self,
        prepared: ModelingDiagnosticPreparation,
        values: CaseValues,
        policy: ModelingDiagnosticPolicy,
        compiler: pse_compiler::workspace::Profile,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingDiagnostics, WorkflowError> {
        policy.validate()?;
        prepared.validate_point(&values)?;
        let ModelingDiagnosticPreparation {
            model: prepared,
            numerics,
            providers,
            route_decision,
            admission_identity,
        } = prepared;
        let product = prepared.model.compiled();
        let plan = &prepared.case.compiled().plan;
        let structure = plan.structure();
        for target in plan
            .numerical_targets(&prepared.case.compiled().quantities)
            .map_err(crate::math::MathRuntimeError::from)?
        {
            if numerics
                .targets
                .iter()
                .find(|t| t.id == target.id && t.kind == target.kind)
                .is_none_or(|t| {
                    t.quantity != target.quantity.as_id()
                        || t.unit != target.unit.as_id()
                        || !t.nominal.is_finite()
                        || t.nominal <= 0.
                })
            {
                return Err(contract(
                    "diagnostics require the case's complete original-unit numerical targets",
                ));
            }
        }
        let extent = structure
            .variables()
            .len()
            .checked_add(structure.rows().len())
            .and_then(|n| n.checked_add(values.scalars.len()))
            .and_then(|n| n.checked_mul(256))
            .and_then(|n| n.checked_add(policy.maximum_findings.checked_mul(1024)?))
            .and_then(|n| n.checked_add(policy.matrix.dense_entries.checked_mul(64)?))
            .ok_or_else(|| contract("diagnostic storage extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("modeling:diagnostic-report", extent)?;
        let nominal = |id, kind| {
            numerics
                .targets
                .iter()
                .find(|t| t.id == id && t.kind == kind)
                .map(|t| t.nominal)
                .ok_or_else(|| contract("diagnostic numerical target absent"))
        };
        let assessment = route_decision
            .structure
            .as_ref()
            .ok_or_else(|| contract("original diagnostic structural assessment absent"))?;
        let equality_count = assessment
            .equations
            .iter()
            .filter(|row| row.lower.is_some() && row.lower == row.upper)
            .count();
        let inequality_count = assessment.equations.len() - equality_count;
        let free_count = assessment.variables.len();
        let mut report = ModelingDiagnostics {
            run_id: pse_operations::mint_id(),
            runtime: self.runtime.clone(),
            source_identity: plan.structure().key(),
            numerical_identity: numerics.key,
            point: values.clone(),
            profile: policy.profile.clone(),
            findings: vec![],
            route_decision,
            admission_identity,
            statistics: BTreeMap::from([
                ("variables".into(), structure.variables().len()),
                (
                    "fixed_variables".into(),
                    structure.variables().iter().filter(|v| v.fixed).count(),
                ),
                ("free_variables".into(), free_count),
                ("parameters".into(), structure.parameters().len()),
                ("equalities".into(), equality_count),
                ("inequalities".into(), inequality_count),
                ("definitions".into(), product.model.instances.len()),
                (
                    "equation_terms".into(),
                    product.admitted.term_outputs.values().map(Vec::len).sum(),
                ),
                ("accumulators".into(), product.model.closures.len()),
            ]),
            matrix: None,
            complete: true,
            rows: structure.rows().iter().map(|r| r.id).collect(),
            columns: plan.columns().to_vec(),
            row_nominals: structure
                .rows()
                .iter()
                .map(|r| nominal(r.id, NumericalTarget::Row).map(|v| (r.id, v)))
                .collect::<Result<_, _>>()?,
            variable_nominals: plan
                .columns()
                .iter()
                .map(|id| nominal(*id, NumericalTarget::Variable).map(|v| (*id, v)))
                .collect::<Result<_, _>>()?,
            _owner: owner,
        };
        let maximum = policy.maximum_findings;
        // Semantic gathers retain source incidence even when algebraic normalization
        // removes a zero coefficient. Observation-only outputs are not active equations.
        let mut equality_variables = std::collections::BTreeSet::new();
        let mut inequality_variables = std::collections::BTreeSet::new();
        let mut objective_variables = std::collections::BTreeSet::new();
        for instance in structure.instances() {
            for contribution in &instance.contributions {
                let set = match contribution.target {
                    pse_math::binding::Target::Objective(_) => &mut objective_variables,
                    pse_math::binding::Target::Row(id) => {
                        let row = structure
                            .rows()
                            .iter()
                            .find(|r| r.id == id)
                            .ok_or_else(|| contract("diagnostic incidence row absent"))?;
                        if row.lower == row.upper {
                            &mut equality_variables
                        } else {
                            &mut inequality_variables
                        }
                    }
                };
                set.extend(instance.slots.iter().map(|s| s.source()));
            }
        }
        let mut counts = BTreeMap::<String, usize>::new();
        for name in [
            "unused_variables",
            "variables_only_in_inequalities",
            "variables_in_equalities",
            "variables_in_inequalities",
            "variables_in_objective",
            "variables_with_lower_bounds",
            "variables_with_upper_bounds",
            "variables_with_both_bounds",
            "variables_without_bounds",
            "missing_values",
            "nonfinite_values",
            "fixed_variables_in_equalities",
            "free_variables_in_equalities",
        ] {
            counts.insert(name.into(), 0);
        }
        for variable in structure.variables() {
            let id = variable.port.id;
            let equality = equality_variables.contains(&id);
            let inequality = inequality_variables.contains(&id);
            let objective = objective_variables.contains(&id);
            for (name, present) in [
                ("unused_variables", !equality && !inequality && !objective),
                ("variables_only_in_inequalities", inequality && !equality),
                ("variables_in_equalities", equality),
                ("variables_in_inequalities", inequality),
                ("variables_in_objective", objective),
                ("variables_with_lower_bounds", variable.lower.is_some()),
                ("variables_with_upper_bounds", variable.upper.is_some()),
                (
                    "variables_with_both_bounds",
                    variable.lower.is_some() && variable.upper.is_some(),
                ),
                (
                    "variables_without_bounds",
                    variable.lower.is_none() && variable.upper.is_none(),
                ),
                ("missing_values", !values.scalars.contains_key(&id)),
                (
                    "nonfinite_values",
                    values.scalars.get(&id).is_some_and(|v| !v.is_finite()),
                ),
                ("fixed_variables_in_equalities", equality && variable.fixed),
                ("free_variables_in_equalities", equality && !variable.fixed),
            ] {
                if present {
                    *counts.entry(name.into()).or_default() += 1;
                }
            }
            if !equality && !inequality && !objective {
                report.push(
                    finding("variable.unused", [id], None, None),
                    maximum,
                    product,
                );
            } else if inequality && !equality {
                report.push(
                    finding("variable.only_in_inequalities", [id], None, None),
                    maximum,
                    product,
                );
            }
        }
        report.statistics.extend(counts);
        let structural = prepared.case.structure();
        for (name, part) in [
            ("structural.underdetermined", &structural.under),
            ("structural.overdetermined", &structural.over),
        ] {
            if !part.rows.is_empty() || !part.columns.is_empty() {
                report.push(
                    finding(
                        name,
                        part.rows.iter().chain(&part.columns).copied(),
                        None,
                        None,
                    ),
                    maximum,
                    product,
                );
            }
        }
        for (id, status) in &prepared.case.compiled().presolve.obligations {
            if *status != pse_math::presolve::ObligationStatus::Discharged {
                let mut f = finding("domain.potential_evaluation_error", [*id], None, None);
                f.observations.insert(
                    "box_status".into(),
                    Observation::Text(status.as_str().into()),
                );
                report.push(f, maximum, product);
            }
        }
        for variable in structure.variables() {
            let id = variable.port.id;
            let Some(value) = values.scalars.get(&id).copied() else {
                report.push(
                    finding("variable.missing_value", [id], None, None),
                    maximum,
                    product,
                );
                continue;
            };
            if !value.is_finite() {
                report.push(
                    finding("variable.nonfinite", [id], Some(value), None),
                    maximum,
                    product,
                );
                continue;
            }
            let magnitude = nominal(id, NumericalTarget::Variable)?;
            let normalized = value.abs() / magnitude;
            if variable.fixed && value.abs() <= policy.variable_zero * magnitude {
                report.push(
                    finding(
                        "variable.fixed_zero",
                        [id],
                        Some(value),
                        Some(policy.variable_zero * magnitude),
                    ),
                    maximum,
                    product,
                );
            }
            if normalized > policy.variable_large {
                report.push(
                    finding(
                        "variable.large_value",
                        [id],
                        Some(normalized),
                        Some(policy.variable_large),
                    ),
                    maximum,
                    product,
                );
            }
            if normalized > policy.variable_zero && normalized < policy.variable_small {
                report.push(
                    finding(
                        "variable.small_value",
                        [id],
                        Some(normalized),
                        Some(policy.variable_small),
                    ),
                    maximum,
                    product,
                );
            }
            for (name, bound, sign) in [
                ("lower", variable.lower, 1.),
                ("upper", variable.upper, -1.),
            ] {
                if let Some(bound) = bound {
                    let distance = sign * (value - bound);
                    let near = policy.near_bound_absolute
                        + policy.near_bound_relative * bound.abs().max(magnitude);
                    if distance < -policy.bound_violation {
                        report.push(
                            finding(
                                &format!("variable.outside_{name}_bound"),
                                [id],
                                Some(-distance),
                                Some(policy.bound_violation),
                            ),
                            maximum,
                            product,
                        );
                    } else if distance <= near {
                        report.push(
                            finding(
                                &format!("variable.near_{name}_bound"),
                                [id],
                                Some(distance),
                                Some(near),
                            ),
                            maximum,
                            product,
                        );
                    }
                }
            }
        }
        let service = self.runtime.shared.math();
        let assembly = service.assemble(prepared.case.clone()).await?;
        let row_ids = structure.rows().iter().map(|r| r.id).collect::<Vec<_>>();
        let columns = plan.columns().to_vec();
        let row_scales = row_ids
            .iter()
            .map(|id| nominal(*id, NumericalTarget::Row).map(|v| 1. / v))
            .collect::<Result<Vec<_>, _>>()?;
        let column_scales = columns
            .iter()
            .map(|id| nominal(*id, NumericalTarget::Variable).map(|v| 1. / v))
            .collect::<Result<Vec<_>, _>>()?;
        let candidate = values.clone();
        let matrix_policy = policy.matrix;
        let scaled_rows = row_scales.clone();
        let scaled_columns = column_scales.clone();
        let retained = report._owner.clone();
        let sampled = service
            .with_worker(assembly, providers.clone(), cancel, move |worker| {
                let rows = worker.constraints(&candidate)?;
                let flag = worker.cancellation().clone();
                let jacobian = worker.jacobian(&candidate)?;
                let entries = (0..jacobian.ncols())
                    .flat_map(|j| {
                        jacobian
                            .row_idx_of_col(j)
                            .zip(jacobian.val_of_col(j))
                            .map(move |(i, v)| (i, j, *v))
                    })
                    .map(|(i, j, v)| (i, j, v * scaled_rows[i] / scaled_columns[j]))
                    .collect::<Vec<_>>();
                let matrix = pse_math::diagnostics::analyze_matrix::<GlobalRow, GlobalCol>(
                    jacobian.as_ref(),
                    &scaled_rows,
                    &scaled_columns,
                    matrix_policy,
                    &flag,
                );
                // A square Jacobian also gets FERAL's sparse 1-norm condition estimate in the
                // same scaling, which needs no dense budget (L-N6).
                let condition = (jacobian.nrows() == jacobian.ncols()).then(|| {
                    let nominals = scaled_columns.iter().map(|s| 1. / s).collect::<Vec<_>>();
                    pse_backend_native::conditioning::jacobian_condition(
                        jacobian.as_ref(),
                        &scaled_rows,
                        &nominals,
                    )
                });
                Ok((rows, entries, matrix, condition, retained))
            })
            .await;
        match sampled {
            Ok((row_values, entries, matrix, condition, _retained)) => {
                for (row, value) in structure.rows().iter().zip(row_values) {
                    let residual = (row.lower - value).max(value - row.upper).max(0.);
                    let scale = nominal(row.id, NumericalTarget::Row)?;
                    if residual > policy.residual * scale {
                        report.push(
                            finding(
                                "equation.large_residual",
                                [row.id],
                                Some(residual),
                                Some(policy.residual * scale),
                            ),
                            maximum,
                            product,
                        );
                    }
                }
                for (i, j, value) in entries {
                    if value.abs() > policy.jacobian_large
                        || value != 0. && value.abs() < policy.jacobian_small
                    {
                        report.push(
                            finding(
                                "jacobian.extreme_entry",
                                [row_ids[i], columns[j]],
                                Some(value),
                                None,
                            ),
                            maximum,
                            product,
                        );
                    }
                }
                match matrix {
                    Ok(matrix) => {
                        for (norms, ids, kind) in [
                            (&matrix.row_norms, &row_ids, "jacobian.extreme_row"),
                            (&matrix.column_norms, &columns, "jacobian.extreme_column"),
                        ] {
                            for (id, value) in ids.iter().zip(norms) {
                                if *value > policy.jacobian_large || *value < policy.jacobian_small
                                {
                                    report.push(
                                        finding(kind, [*id], Some(*value), None),
                                        maximum,
                                        product,
                                    );
                                }
                            }
                        }
                        let (rows, cols): (
                            &TiSlice<GlobalRow, SemanticId>,
                            &TiSlice<GlobalCol, SemanticId>,
                        ) = (row_ids.as_slice().as_ref(), columns.as_slice().as_ref());
                        let parallel = matrix
                            .parallel_rows
                            .iter()
                            .map(|p| {
                                (
                                    "jacobian.parallel_rows",
                                    [rows[p.first], rows[p.second]],
                                    p.cosine,
                                )
                            })
                            .chain(matrix.parallel_columns.iter().map(|p| {
                                (
                                    "jacobian.parallel_columns",
                                    [cols[p.first], cols[p.second]],
                                    p.cosine,
                                )
                            }))
                            .collect::<Vec<_>>();
                        for (kind, ids, cosine) in parallel {
                            report.push(
                                finding(
                                    kind,
                                    ids,
                                    Some(cosine),
                                    Some(policy.matrix.parallel_tolerance),
                                ),
                                maximum,
                                product,
                            );
                        }
                        if matrix.rank < row_ids.len().min(columns.len()) {
                            // The deficiency names the equations and variables that carry
                            // its near-null modes: those whose component of a left or right
                            // singular vector below the cutoff exceeds the profile's
                            // singular-vector threshold.
                            let threshold = policy.matrix.singular_vector;
                            let mut members = std::collections::BTreeSet::new();
                            for mode in matrix.modes.iter().filter(|m| m.value <= matrix.cutoff) {
                                members.extend(
                                    mode.left
                                        .iter()
                                        .zip(row_ids.iter())
                                        .filter(|(v, _)| v.abs() > threshold)
                                        .map(|(_, id)| *id),
                                );
                                members.extend(
                                    mode.right
                                        .iter()
                                        .zip(columns.iter())
                                        .filter(|(v, _)| v.abs() > threshold)
                                        .map(|(_, id)| *id),
                                );
                            }
                            report.push(
                                finding(
                                    "jacobian.numerical_rank_deficiency",
                                    members,
                                    Some(matrix.rank as f64),
                                    Some(matrix.cutoff),
                                ),
                                maximum,
                                product,
                            );
                        }
                        report.matrix = Some(matrix);
                    }
                    Err(error) => {
                        report.complete = false;
                        let mut f = BoundaryDiagnostic::new(
                            BoundaryClass::Inconclusive,
                            "modeling.diagnostics",
                            [],
                            "jacobian.analysis_inconclusive",
                        )
                        .with_severity(Severity::Warning);
                        f.observations
                            .insert("reason".into(), Observation::Text(error.to_string()));
                        report.push(f, maximum, product);
                    }
                }
                let locations = || row_ids.iter().chain(&columns).copied();
                match condition {
                    Some(Ok(Some(estimate))) => report.push(
                        finding(
                            "jacobian.condition_estimate",
                            locations(),
                            Some(estimate),
                            None,
                        ),
                        maximum,
                        product,
                    ),
                    Some(Ok(None)) => {
                        let mut f = finding("jacobian.condition_estimate", locations(), None, None);
                        f.observations.insert(
                            "reason".into(),
                            Observation::Text("singular at the sparse LU pivot tolerance".into()),
                        );
                        report.push(f, maximum, product);
                    }
                    Some(Err(error)) => {
                        report.complete = false;
                        let mut f = BoundaryDiagnostic::new(
                            BoundaryClass::Inconclusive,
                            "modeling.diagnostics",
                            [],
                            "jacobian.analysis_inconclusive",
                        )
                        .with_severity(Severity::Warning);
                        f.observations
                            .insert("reason".into(), Observation::Text(error.to_string()));
                        report.push(f, maximum, product);
                    }
                    None => {}
                }
            }
            Err(error) => {
                report.complete = false;
                let mut f = BoundaryDiagnostic::new(
                    BoundaryClass::TrialRejected,
                    "modeling.diagnostics",
                    [],
                    "candidate.evaluation_failed",
                );
                f.observations
                    .insert("reason".into(), Observation::Text(error.to_string()));
                report.push(f, maximum, product);
            }
        }
        let terms = product
            .admitted
            .term_outputs
            .values()
            .flatten()
            .map(|(id, _)| *id)
            .collect();
        match self
            .observe_registered(
                prepared.model.clone(),
                terms,
                values,
                compiler,
                providers,
                cancel,
            )
            .await
        {
            Ok(observed) => {
                let magnitudes = product
                    .admitted
                    .term_outputs
                    .iter()
                    .map(|(equation, terms)| {
                        (
                            *equation,
                            terms.iter().map(|(id, sign)| sign * observed[id]).collect(),
                        )
                    })
                    .collect::<BTreeMap<_, Vec<_>>>();
                let evidence = service
                    .modeling_term_diagnostics(magnitudes.clone(), policy.terms, cancel)
                    .await?;
                if evidence.unattempted > 0 {
                    report.complete = false;
                }
                for (equation, result) in &evidence.results {
                    let terms = &product.admitted.term_outputs[equation];
                    if !result.complete {
                        report.complete = false;
                    }
                    for index in &result.mismatched {
                        report.push(
                            finding(
                                "equation.mismatched_term",
                                [*equation, terms[*index].0],
                                Some(magnitudes[equation][*index]),
                                Some(policy.terms.mismatch),
                            ),
                            maximum,
                            product,
                        );
                    }
                    for set in &result.cancellations {
                        report.push(
                            finding(
                                "equation.canceling_terms",
                                std::iter::once(*equation).chain(set.iter().map(|i| terms[*i].0)),
                                None,
                                Some(policy.terms.cancellation),
                            ),
                            maximum,
                            product,
                        );
                    }
                }
            }
            Err(error) => {
                report.complete = false;
                let mut f = finding("equation.term_evaluation_failed", [], None, None);
                f.observations
                    .insert("reason".into(), Observation::Text(error.to_string()));
                report.push(f, maximum, product);
            }
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_compiler::workspace::ModelingCaseBindings;
    use pse_kernels::DerivativeOrder;
    #[tokio::test]
    async fn kernel_diagnostics_missing_starts_remain_missing_and_solves_stay_strict() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let declarations = pse_authoring::language::parse(
            "package p { def Root {var x:Scalar; eq e:x==2; annotation bounds x(0,1);} }",
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
        let package = rt.modeling_package(declarations, physical).unwrap();
        let cancel = crate::CancelSource::new();
        let mut analysis = ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Bindings::default(),
            limits: Limits::default(),
            case: Default::default(),
            order: DerivativeOrder::First,
            compiler: super::super::super::tests::compiler_profile(),
            solver: super::super::super::tests::profile(),
            numerical: Default::default(),
        };
        let prepared = package
            .prepare_diagnostics(&analysis, &cancel)
            .await
            .unwrap();
        assert!(prepared.model.values.scalars.is_empty());
        let key = prepared.model.case.compiled().plan.structure().key();
        let policy: ModelingDiagnosticPolicy = serde_json::from_str(include_str!(
            "../../../../../packages/reference/diagnostics/idaes-2.13.json"
        ))
        .unwrap();
        let report = package
            .diagnose_case(
                prepared.clone(),
                prepared.model.values.clone(),
                policy.clone(),
                analysis.compiler,
                &cancel,
            )
            .await
            .unwrap();
        assert!(!report.complete);
        assert_eq!(report.statistics["missing_values"], 1);
        assert!(report.point.scalars.is_empty());
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.rule == "variable.missing_value")
        );
        assert!(
            package
                .prepare_analysis_attempt(&analysis, Default::default(), &cancel)
                .await
                .is_err()
        );
        analysis.case.values.insert("x".into(), 2.);
        let prepared = package
            .prepare_diagnostics(&analysis, &cancel)
            .await
            .unwrap();
        assert_eq!(key, prepared.model.case.compiled().plan.structure().key());
        let report = package
            .diagnose_case(
                prepared.clone(),
                prepared.model.values.clone(),
                policy,
                analysis.compiler,
                &cancel,
            )
            .await
            .unwrap();
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.rule == "variable.outside_upper_bound")
        );
        assert!(
            package
                .prepare_analysis_attempt(&analysis, Default::default(), &cancel)
                .await
                .is_err()
        );
        analysis.case.values.clear();
        analysis.case.variables.insert(
            "x".into(),
            pse_compiler::workspace::ModelingVariableState {
                fixed: Some(true),
                ..Default::default()
            },
        );
        assert!(
            package
                .prepare_diagnostics(&analysis, &cancel)
                .await
                .unwrap_err()
                .to_string()
                .contains("missing frozen")
        );
    }
    #[tokio::test]
    async fn kernel_diagnostics_name_sources_and_keep_numerical_rank_distinct_from_structure() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let rows = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; var y:Scalar; eq a:x+y==0; eq b:2*x+2*y==0; } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical).unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let analysis = ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Bindings::default(),
            limits: Limits::default(),
            case: ModelingCaseBindings {
                values: BTreeMap::from([("x".into(), 1.), ("y".into(), -1.)]),
                ..Default::default()
            },
            order: DerivativeOrder::First,
            compiler,
            solver: super::super::super::tests::profile(),
            numerical: Default::default(),
        };
        let prepared = package
            .prepare_diagnostics(&analysis, &cancel)
            .await
            .unwrap();
        assert_eq!(prepared.model.case.structure().matching.len(), 2);
        let values = prepared.model.values.clone();
        let p = ModelingDiagnosticPolicy {
            profile: "synthetic".into(),
            maximum_findings: 50,
            near_bound_absolute: 1e-4,
            near_bound_relative: 1e-4,
            bound_violation: 0.,
            residual: 1e-5,
            variable_small: 1e-4,
            variable_large: 1e4,
            variable_zero: 1e-8,
            jacobian_small: 1e-4,
            jacobian_large: 1e4,
            matrix: MatrixPolicy {
                dense_entries: 100,
                findings: 50,
                parallel_tolerance: 1e-8,
                rank_absolute: 1e-12,
                rank_relative: 1e-8,
                singular_vector: 0.1,
            },
            terms: TermPolicy {
                zero: 1e-10,
                mismatch: 1e6,
                cancellation: 1e-4,
                maximum_terms: 5,
                combinations: 100,
                findings: 50,
            },
        };
        let report = package
            .diagnose_case(
                prepared.clone(),
                values.clone(),
                p.clone(),
                compiler,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(report.matrix.as_ref().unwrap().rank, 1);
        assert!(report.complete);
        let points = vec![
            (
                pse_ids::named_id(root.as_id(), "sample-good"),
                values.clone(),
            ),
            (
                pse_ids::named_id(root.as_id(), "sample-missing"),
                CaseValues::default(),
            ),
            (
                pse_ids::named_id(root.as_id(), "sample-recovered"),
                values.clone(),
            ),
        ];
        let samples = package
            .diagnose_samples(
                prepared.clone(),
                points.clone(),
                p.clone(),
                compiler,
                3,
                std::time::Duration::from_secs(10),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(samples.stop, DiagnosticSampleStop::Completed);
        assert_eq!(samples.unattempted, 0);
        assert!(samples.outcomes[0].1.as_ref().unwrap().complete);
        assert!(samples.outcomes[1].1.as_ref().map_or(true, |r| !r.complete));
        assert!(samples.outcomes[2].1.as_ref().unwrap().complete);
        let samples = package
            .diagnose_samples(
                prepared.clone(),
                points.clone(),
                p.clone(),
                compiler,
                1,
                std::time::Duration::from_secs(10),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(samples.stop, DiagnosticSampleStop::SampleLimit);
        assert_eq!(samples.unattempted, 2);
        let stopped = crate::CancelSource::new();
        stopped.cancel();
        let samples = package
            .diagnose_samples(
                prepared.clone(),
                points,
                p.clone(),
                compiler,
                3,
                std::time::Duration::from_secs(10),
                &stopped,
            )
            .await
            .unwrap();
        assert_eq!(samples.stop, DiagnosticSampleStop::Cancelled);
        assert_eq!(samples.unattempted, 3);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.rule == "jacobian.parallel_rows" && f.locations.len() == 2)
        );
        // The square Jacobian [[1, 1], [2, 2]] is singular for the sparse LU as well.
        let condition = report
            .findings
            .iter()
            .find(|f| f.rule == "jacobian.condition_estimate")
            .unwrap();
        assert!(!condition.observations.contains_key("value"));
        assert!(condition.observations.contains_key("reason"));
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.rule == "equation.canceling_terms")
        );
        assert!(
            !report
                .findings
                .iter()
                .any(|f| f.rule.starts_with("structural."))
        );
        #[cfg(feature = "solver-kinsol")]
        {
            let physical = super::super::super::tests::physical();
            let rows = pse_authoring::language::parse("package p { def Root { var x:Scalar; var unused:Scalar; implicit i {var y:Scalar; eq e:y==2; annotation start y(1);} realize r on i using nested; eq e:x==i.y; } }", SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, pse_authoring::ParseBudget::default()).unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let nested = rt.modeling_package(rows, physical).unwrap();
            let analysis = ModelingAnalysis {
                root,
                instance: pse_modeling::specialize::root_instance(root),
                case: ModelingCaseBindings {
                    values: BTreeMap::from([("x".into(), 2.), ("unused".into(), 1.)]),
                    ..Default::default()
                },
                ..analysis
            };
            let prepared = nested
                .prepare_diagnostics(&analysis, &cancel)
                .await
                .unwrap();
            assert_eq!(prepared.providers.len(), 1);
            let values = prepared.model.values.clone();
            let report = nested
                .diagnose_case(prepared, values, p, compiler, &cancel)
                .await
                .unwrap();
            assert!(report.complete, "{:?}", report.findings);
            assert_eq!(report.matrix.as_ref().unwrap().rank, 1);
            assert!(
                report
                    .findings
                    .iter()
                    .any(|f| f.rule == "structural.underdetermined")
            );
            assert_eq!(report.statistics["unused_variables"], 1);
            assert!(report.findings.iter().any(|f| f.rule == "variable.unused"));
        }
        #[cfg(feature = "solver-highs")]
        {
            let native = package
                .diagnose_jacobian_optimization(
                    prepared,
                    values,
                    pse_backend_native::jacobian_diagnostics::Policy {
                        maximum_rows: 10,
                        maximum_entries: 1000,
                        maximum_attempts: 4,
                        multiplier_bound: 10.,
                        tolerance: 1e-7,
                        rank_relative: 1e-8,
                        maximum_nodes: None,
                    },
                    pse_backend_native::solve::Controls::default(),
                    &cancel,
                )
                .await
                .unwrap();
            assert!(native.evidence.complete);
            assert_eq!(native.evidence.degenerate.len(), 1);
            assert!(native.evidence.degenerate[0].irreducible_at_tolerance);
            let exported = native.into_export().unwrap();
            assert_eq!(exported.table.batch().num_rows(), 1);
            assert_eq!(exported.attempts.len(), 4);
        }
    }
}

/// Bounded LP/MILP analyses of one diagnosed Jacobian.
#[cfg(feature = "solver-highs")]
#[derive(Debug)]
pub struct ModelingJacobianOptimization {
    /// Identity of this analysis run.
    pub run_id: RunId,
    /// Certificates, degenerate sets and every native attempt.
    pub evidence: pse_backend_native::jacobian_diagnostics::Report,
    pub(in crate::workflow::modeling) runtime: Runtime,
    pub(in crate::workflow::modeling) source_identity: pse_ids::ContentHash,
    pub(in crate::workflow::modeling) numerical_identity: pse_ids::ContentHash,
    pub(in crate::workflow::modeling) point: CaseValues,
    pub(in crate::workflow::modeling) row_nominals: Vec<(SemanticId, f64)>,
    pub(in crate::workflow::modeling) variable_nominals: Vec<(SemanticId, f64)>,
    pub(in crate::workflow::modeling) policy: pse_backend_native::jacobian_diagnostics::Policy,
    pub(in crate::workflow::modeling) _owner: Arc<pse_columnar::AllocationLease>,
}
#[cfg(feature = "solver-highs")]
impl ModelingPackage {
    /// Separate bounded native analyses of a scaled Jacobian, under one admitted worker.
    pub async fn diagnose_jacobian_optimization(
        &self,
        prepared: ModelingDiagnosticPreparation,
        values: CaseValues,
        policy: pse_backend_native::jacobian_diagnostics::Policy,
        controls: pse_backend_native::solve::Controls,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingJacobianOptimization, WorkflowError> {
        prepared.validate_point(&values)?;
        let ModelingDiagnosticPreparation {
            model: prepared,
            numerics,
            providers,
            route_decision: _,
            admission_identity: _,
        } = prepared;
        let plan = &prepared.case.compiled().plan;
        let rows = plan
            .structure()
            .rows()
            .iter()
            .map(|r| r.id)
            .collect::<Vec<_>>();
        let normalization =
            pse_math::normalization::Normalization::from_policy(&numerics, plan.columns(), &rows)
                .map_err(crate::math::MathRuntimeError::from)?;
        controls
            .validate()
            .map_err(crate::math::MathRuntimeError::from)?;
        let source_identity = plan.structure().key();
        let numerical_identity = numerics.key;
        let allowance = controls
            .report_allowance()
            .map_err(crate::math::MathRuntimeError::from)?
            .checked_mul(policy.maximum_attempts)
            .and_then(|n| n.checked_add(policy.maximum_entries.checked_mul(128)?))
            .and_then(|n| n.checked_add(values.scalars.len().checked_mul(128)?))
            .ok_or_else(|| contract("native diagnostic extent"))?;
        let service = self.runtime.shared.math();
        let owner = service.reserve("modeling:native-diagnostics", allowance)?;
        let point = values.clone();
        let row_nominals = rows
            .iter()
            .copied()
            .zip(normalization.rows.iter().copied())
            .collect();
        let variable_nominals = plan
            .columns()
            .iter()
            .copied()
            .zip(normalization.variables.iter().copied())
            .collect();
        let retained = owner.clone();
        let assembly = service.assemble(prepared.case).await?;
        let (evidence, owner) = service
            .with_worker(assembly, providers, cancel, move |worker| {
                let flag = worker.cancellation().clone();
                let original = worker.jacobian(&values)?;
                let mut normalized = original.clone();
                for j in 0..normalized.ncols() {
                    let indices = normalized.row_idx_of_col(j).collect::<Vec<_>>();
                    for (i, v) in indices.into_iter().zip(normalized.val_of_col_mut(j)) {
                        *v = *v * normalization.variables[j] / normalization.rows[i];
                    }
                }
                let evidence = pse_backend_native::jacobian_diagnostics::analyze(
                    normalized.as_ref(),
                    &rows,
                    policy,
                    &controls,
                    pse_backend_native::solve::Execution::new(flag, &controls),
                )?;
                Ok((evidence, retained))
            })
            .await?;
        Ok(ModelingJacobianOptimization {
            run_id: pse_operations::mint_id(),
            runtime: self.runtime.clone(),
            source_identity,
            numerical_identity,
            point,
            row_nominals,
            variable_nominals,
            policy,
            evidence,
            _owner: owner,
        })
    }
}
