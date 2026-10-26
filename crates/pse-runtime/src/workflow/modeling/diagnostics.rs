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
use pse_compiler::workspace::ModelingOutput;
use pse_model::generated::identities::RunId;
#[path = "diagnostic_nonlinear.rs"]
mod nonlinear;
#[path = "diagnostic_samples.rs"]
mod samples;
#[path = "diagnostic_source_map.rs"]
mod source_map;
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
pub use source_map::{ModelingSourceMap, ModelingSourceMapDocument};
use std::sync::Arc;

/// Diagnostic preparation resolves the same case and native dependencies without solver admission.
#[derive(Clone, Debug)]
pub struct ModelingDiagnosticPreparation {
    /// The prepared case, with its structure and start values.
    pub model: ModelingCasePreparation,
    /// Resolved physical nominals and numerical requirements of the case.
    pub numerics: pse_math::SharedAllocation<ResolvedNumericalPolicy>,
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
    /// Retained original invocation, when this is follow-up analysis.
    /// Retained original invocation or failure, independent of scientific permission.
    pub origin: Option<super::super::DiagnosticOrigin>,
    pub(super) encodings: Arc<crate::workflow::result_export::EncodingState>,
    /// Identity of this diagnostic run.
    pub run_id: RunId,
    pub(super) services: crate::workflow::PreparationServices,
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
#[derive(Clone, Copy)]
enum FindingKind {
    DomainPotentialEvaluationError,
    EquationCancelingTerms,
    EquationLargeResidual,
    EquationMismatchedTerm,
    EquationTermEvaluationFailed,
    JacobianConditionEstimate,
    JacobianExtremeColumn,
    JacobianExtremeEntry,
    JacobianExtremeRow,
    JacobianNumericalRankDeficiency,
    JacobianParallelColumns,
    JacobianParallelRows,
    StructuralOverdetermined,
    StructuralUnderdetermined,
    VariableFixedZero,
    VariableLargeValue,
    VariableMissingValue,
    VariableNearLowerBound,
    VariableNearUpperBound,
    VariableNonfinite,
    VariableOnlyInInequalities,
    VariableOutsideLowerBound,
    VariableOutsideUpperBound,
    VariableSmallValue,
    VariableUnused,
}
impl FindingKind {
    fn rule(self) -> pse_diagnostics::DiagnosticRule {
        match self {
            Self::DomainPotentialEvaluationError => {
                pse_diagnostics::DiagnosticRule::DomainPotentialEvaluationError
            }
            Self::EquationCancelingTerms => pse_diagnostics::DiagnosticRule::EquationCancelingTerms,
            Self::EquationLargeResidual => pse_diagnostics::DiagnosticRule::EquationLargeResidual,
            Self::EquationMismatchedTerm => pse_diagnostics::DiagnosticRule::EquationMismatchedTerm,
            Self::EquationTermEvaluationFailed => {
                pse_diagnostics::DiagnosticRule::EquationTermEvaluationFailed
            }
            Self::JacobianConditionEstimate => {
                pse_diagnostics::DiagnosticRule::JacobianConditionEstimate
            }
            Self::JacobianExtremeColumn => pse_diagnostics::DiagnosticRule::JacobianExtremeColumn,
            Self::JacobianExtremeEntry => pse_diagnostics::DiagnosticRule::JacobianExtremeEntry,
            Self::JacobianExtremeRow => pse_diagnostics::DiagnosticRule::JacobianExtremeRow,
            Self::JacobianNumericalRankDeficiency => {
                pse_diagnostics::DiagnosticRule::JacobianNumericalRankDeficiency
            }
            Self::JacobianParallelColumns => {
                pse_diagnostics::DiagnosticRule::JacobianParallelColumns
            }
            Self::JacobianParallelRows => pse_diagnostics::DiagnosticRule::JacobianParallelRows,
            Self::StructuralOverdetermined => {
                pse_diagnostics::DiagnosticRule::StructuralOverdetermined
            }
            Self::StructuralUnderdetermined => {
                pse_diagnostics::DiagnosticRule::StructuralUnderdetermined
            }
            Self::VariableFixedZero => pse_diagnostics::DiagnosticRule::VariableFixedZero,
            Self::VariableLargeValue => pse_diagnostics::DiagnosticRule::VariableLargeValue,
            Self::VariableMissingValue => pse_diagnostics::DiagnosticRule::VariableMissingValue,
            Self::VariableNearLowerBound => pse_diagnostics::DiagnosticRule::VariableNearLowerBound,
            Self::VariableNearUpperBound => pse_diagnostics::DiagnosticRule::VariableNearUpperBound,
            Self::VariableNonfinite => pse_diagnostics::DiagnosticRule::VariableNonfinite,
            Self::VariableOnlyInInequalities => {
                pse_diagnostics::DiagnosticRule::VariableOnlyInInequalities
            }
            Self::VariableOutsideLowerBound => {
                pse_diagnostics::DiagnosticRule::VariableOutsideLowerBound
            }
            Self::VariableOutsideUpperBound => {
                pse_diagnostics::DiagnosticRule::VariableOutsideUpperBound
            }
            Self::VariableSmallValue => pse_diagnostics::DiagnosticRule::VariableSmallValue,
            Self::VariableUnused => pse_diagnostics::DiagnosticRule::VariableUnused,
        }
    }
}
fn disposition(kind: FindingKind) -> (BoundaryClass, Severity) {
    use BoundaryClass as C;
    use Severity as S;
    match kind {
        FindingKind::StructuralUnderdetermined
        | FindingKind::StructuralOverdetermined
        | FindingKind::VariableMissingValue => (C::InvalidModel, S::Error),
        FindingKind::VariableNonfinite => (C::Nonfinite, S::Error),
        FindingKind::VariableUnused
        | FindingKind::VariableOnlyInInequalities
        | FindingKind::DomainPotentialEvaluationError => (C::InvalidModel, S::Info),
        FindingKind::VariableOutsideLowerBound
        | FindingKind::VariableOutsideUpperBound
        | FindingKind::EquationLargeResidual
        | FindingKind::JacobianParallelRows
        | FindingKind::JacobianParallelColumns
        | FindingKind::JacobianNumericalRankDeficiency => (C::Numerical, S::Warning),
        FindingKind::VariableNearLowerBound
        | FindingKind::VariableNearUpperBound
        | FindingKind::VariableFixedZero
        | FindingKind::VariableLargeValue
        | FindingKind::VariableSmallValue
        | FindingKind::JacobianExtremeEntry
        | FindingKind::JacobianExtremeRow
        | FindingKind::JacobianExtremeColumn
        | FindingKind::JacobianConditionEstimate
        | FindingKind::EquationMismatchedTerm
        | FindingKind::EquationCancelingTerms => (C::Numerical, S::Info),
        FindingKind::EquationTermEvaluationFailed => (C::TrialRejected, S::Warning),
    }
}
fn finding(
    kind: FindingKind,
    ids: impl IntoIterator<Item = SemanticId>,
    score: Option<f64>,
    threshold: Option<f64>,
) -> BoundaryDiagnostic {
    let (class, severity) = disposition(kind);
    let mut f = BoundaryDiagnostic::new(
        class,
        pse_diagnostics::DiagnosticStage::ModelingDiagnostics,
        ids,
        kind.rule(),
    )
    .with_severity(severity);
    if let Some(v) = score {
        f.observations
            .insert("value".into(), Observation::number(v));
    }
    if let Some(v) = threshold {
        f.observations
            .insert("threshold".into(), Observation::number(v));
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
/// Resolve only compiler-supplied original contributors. This is a source
/// projection, never a numerical pullback or an instruction to re-evaluate a row.
fn original_contributors(
    model: &pse_compiler::workspace::PreparedModeling,
    id: SemanticId,
) -> std::collections::BTreeSet<SemanticId> {
    let mut sources = std::collections::BTreeSet::from([id]);
    sources.extend(
        model
            .model
            .ports
            .values()
            .filter(|port| port.symbol == id)
            .map(|port| port.id),
    );
    sources.extend(
        model
            .model
            .material_ports
            .values()
            .filter(|port| {
                port.coordinates
                    .values()
                    .any(|coordinate| *coordinate == id)
            })
            .map(|port| port.id),
    );
    for output in model
        .admitted
        .outputs
        .iter()
        .filter(|output| output.row_id() == id)
    {
        match output {
            ModelingOutput::DynamicRate { state, equations } => {
                sources.insert(*state);
                sources.extend(equations);
            }
            ModelingOutput::InitialState { state, equation } => {
                sources.extend([*state, *equation]);
            }
            ModelingOutput::Contribution {
                accumulator,
                contribution,
            } => {
                sources.extend([*accumulator, *contribution]);
            }
            ModelingOutput::InventoryTransfer { balance, event } => {
                sources.extend([*balance, *event]);
            }
            ModelingOutput::Hint {
                target,
                declaration,
                ..
            } => {
                sources.extend([*target, declaration.as_id()]);
            }
            ModelingOutput::Term { equation, .. } => {
                sources.insert(*equation);
            }
            ModelingOutput::OriginalEquation(source)
            | ModelingOutput::Penalty(source)
            | ModelingOutput::Member(source)
            | ModelingOutput::Inventory(source)
            | ModelingOutput::ConditionalBoundary(source) => {
                sources.insert(*source);
            }
            ModelingOutput::Test { id, .. } | ModelingOutput::Equation { id, .. } => {
                sources.insert(*id);
            }
            ModelingOutput::LevelBound { row, .. } => {
                for level in &model.model.objectives.levels {
                    if level.bound.as_ref().is_some_and(|bound| bound.row == *row) {
                        for member in model.model.objectives.members_of(level) {
                            sources.extend([
                                member.target,
                                member.term,
                                member.lineage.declaration.as_id(),
                            ]);
                        }
                    }
                }
            }
        }
    }
    sources
}
/// Attach every supplied authored lineage and exact current field occurrence,
/// including aliases and nested causes. Byte ranges always retain their document.
pub(super) fn attribute(
    diagnostic: &mut BoundaryDiagnostic,
    model: &pse_compiler::workspace::PreparedModeling,
) {
    let sources = diagnostic
        .sources
        .iter()
        .flat_map(|&id| original_contributors(model, id))
        .collect::<std::collections::BTreeSet<_>>();
    diagnostic.sources = sources.iter().copied().collect();
    let occurrences = model.occurrence_identity();
    let mut add = |source: SemanticId, lineage: &pse_modeling::specialize::Lineage| {
        let fields = occurrences
            .iter()
            .filter(|(declaration, _, _, _)| *declaration == lineage.declaration)
            .collect::<Vec<_>>();
        let mut append = |document, role, position, start, end| {
            if diagnostic.locations.iter().any(|location| {
                location.source == source
                    && location.path == lineage.path
                    && location.document == document
                    && location.occurrence_role == role
                    && location.occurrence_position == position
                    && location.start == start
                    && location.end == end
            }) {
                return;
            }
            diagnostic.locations.push(SourceLocation {
                source,
                revision: None,
                document,
                occurrence_role: role,
                occurrence_position: position,
                path: lineage.path.clone(),
                name: None,
                start,
                end,
            });
        };
        if fields.is_empty() {
            append(None, None, None, None, None);
        }
        for (_, role, position, span) in fields {
            let document = (span.document_id != SemanticId::NIL).then_some(span.document_id);
            let ranged = document.is_some() && !span.is_empty();
            append(
                document,
                Some(role.clone()),
                u32::try_from(*position).ok(),
                ranged.then_some(span.start),
                ranged.then_some(span.end),
            );
        }
    };
    for id in sources {
        if let Some(symbol) = model.model.symbols.get(&id) {
            add(id, &symbol.lineage);
        }
        for row in model.model.equations.iter().filter(|row| row.id == id) {
            add(id, &row.lineage);
        }
        for port in model
            .model
            .ports
            .values()
            .filter(|port| port.id == id || port.symbol == id)
        {
            add(port.id, &port.lineage);
        }
        for port in model.model.material_ports.values().filter(|port| {
            port.id == id
                || port
                    .coordinates
                    .values()
                    .any(|coordinate| *coordinate == id)
        }) {
            add(port.id, &port.lineage);
        }
        for balance in model.model.inventory_balances.values().filter(|balance| {
            balance.id == id
                || balance.inventory_id == id
                || balance.flux_id == id
                || balance.state == Some(id)
        }) {
            add(balance.id, &balance.lineage);
        }
        for condition in model
            .model
            .inventory_initial_conditions
            .values()
            .filter(|condition| condition.row == id)
        {
            add(id, &condition.lineage);
        }
        for lowering in model
            .model
            .lowerings
            .iter()
            .filter(|lowering| lowering.rows.contains(&id) || lowering.variables.contains(&id))
        {
            // The retained owner instance prevents a shared form definition from
            // falsely attributing every other instance that uses that definition.
            for symbol in model.model.symbols.values().filter(|symbol| {
                symbol.lineage.declaration == lowering.source
                    && symbol.lineage.instance == lowering.instance
            }) {
                add(id, &symbol.lineage);
            }
            for row in model.model.equations.iter().filter(|row| {
                row.lineage.declaration == lowering.source
                    && row.lineage.instance == lowering.instance
            }) {
                add(id, &row.lineage);
            }
        }
    }
    for cause in &mut diagnostic.causes {
        attribute(cause, model);
    }
}
impl ModelingPackage {
    /// Resolve starts, specifications, source precedence and nested providers without requesting a solve.
    pub async fn prepare_diagnostics(
        &self,
        services: &crate::workflow::PreparationServices,
        analysis: &ModelingAnalysis,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingDiagnosticPreparation, WorkflowError> {
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let resolved = self
                    .resolve_case(
                        services,
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
                self.finish_diagnostics(services, resolved)
            })
            .await
    }
    /// Inspect this exact retained declaration with diagnostic missing-free policy,
    /// independently of the authored execution procedure.
    pub async fn prepare_declared_diagnostics(
        &self,
        services: &crate::workflow::PreparationServices,
        execution: &DeclaredExecution,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingDiagnosticPreparation, WorkflowError> {
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let model = execution.prepared_model(self)?;
                let analysis = &execution.analysis;
                let resolved = self
                    .resolve_prepared_case(
                        services,
                        model,
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
                self.finish_diagnostics(services, resolved)
            })
            .await
    }
    fn finish_diagnostics(
        &self,
        services: &crate::workflow::PreparationServices,
        resolved: cases::ModelingCaseResolution,
    ) -> Result<ModelingDiagnosticPreparation, WorkflowError> {
        let mut route_decision = resolved.route_decision()?;
        if route_decision.structure.is_none() {
            // Diagnostics retain the requested candidate's original witness even
            // when it cannot admit a solve. Other candidates may interpret that
            // same witness differently, so no unnamed alternative is substituted.
            let backend = match route_decision.selection {
                pse_backend_native::solve::SolverSelection::Explicit(backend) => Some(backend),
                pse_backend_native::solve::SolverSelection::Auto => route_decision.pending_backend,
            };
            route_decision.structure = backend.and_then(|backend| {
                route_decision
                    .eligibility
                    .iter()
                    .find(|candidate| candidate.backend == backend)
                    .and_then(|candidate| candidate.structure.clone())
            });
        }
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
        let numerics = services.math().policy_resolution_owner().resolve(
            physical.clone(),
            resolved.model.case.compiled().preconditions.clone(),
            &targets,
            &resolved.numerical.declarations,
            &resolved.solver.numerics,
        )?;
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
        services: &crate::workflow::ExecutionServices,
        prepared: ModelingDiagnosticPreparation,
        values: CaseValues,
        policy: ModelingDiagnosticPolicy,
        compiler: pse_compiler::workspace::Profile,
        operation: &super::super::DiagnosticOperation,
    ) -> Result<ModelingDiagnostics, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let activity = services.preparation().admit_activity()?;
        activity.scope(async {
        operation.check()?;
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
            .and_then(|n| n.checked_add(policy.matrix.dense_entries.checked_mul(64)?)
            .and_then(|n| n.checked_add(operation.retained_bytes())))
            .ok_or_else(|| contract("diagnostic storage extent"))?;
        let owner = services.preparation()
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
                    services: services.preparation().clone(),
            origin: operation.origin.clone(),
            encodings: Arc::default(),
            run_id: pse_operations::mint_id(),

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
                    finding(FindingKind::VariableUnused, [id], None, None),
                    maximum,
                    product,
                );
            } else if inequality && !equality {
                report.push(
                    finding(FindingKind::VariableOnlyInInequalities, [id], None, None),
                    maximum,
                    product,
                );
            }
        }
        report.statistics.extend(counts);
        let structural = prepared.case.structure();
        for (name, part) in [
            (FindingKind::StructuralUnderdetermined, &structural.under),
            (FindingKind::StructuralOverdetermined, &structural.over),
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
                let mut f = finding(
                    FindingKind::DomainPotentialEvaluationError,
                    [*id],
                    None,
                    None,
                );
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
                    finding(FindingKind::VariableMissingValue, [id], None, None),
                    maximum,
                    product,
                );
                continue;
            };
            if !value.is_finite() {
                report.push(
                    finding(FindingKind::VariableNonfinite, [id], Some(value), None),
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
                        FindingKind::VariableFixedZero,
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
                        FindingKind::VariableLargeValue,
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
                        FindingKind::VariableSmallValue,
                        [id],
                        Some(normalized),
                        Some(policy.variable_small),
                    ),
                    maximum,
                    product,
                );
            }
            for (bound, sign, outside, near_bound) in [
                (
                    variable.lower,
                    1.,
                    FindingKind::VariableOutsideLowerBound,
                    FindingKind::VariableNearLowerBound,
                ),
                (
                    variable.upper,
                    -1.,
                    FindingKind::VariableOutsideUpperBound,
                    FindingKind::VariableNearUpperBound,
                ),
            ] {
                if let Some(bound) = bound {
                    let distance = sign * (value - bound);
                    let near = policy.near_bound_absolute
                        + policy.near_bound_relative * bound.abs().max(magnitude);
                    if distance < -policy.bound_violation {
                        report.push(
                            finding(outside, [id], Some(-distance), Some(policy.bound_violation)),
                            maximum,
                            product,
                        );
                    } else if distance <= near {
                        report.push(
                            finding(near_bound, [id], Some(distance), Some(near)),
                            maximum,
                            product,
                        );
                    }
                }
            }
        }
        let service = services.preparation().math();
        let assembly = crate::math::MathService::within_task(operation.scope(), operation.driver(), service.assemble(prepared.case.clone())).await?;
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
            .with_owned_worker(assembly, providers.clone(), operation.driver(), Some((operation.scope().clone(), operation.scope().cancellation_control().clone())), move |mut worker| {
                let rows = crate::workflow::dynamics::analysis("diagnostic.original_rows", || worker.constraints(&candidate).map(<[f64]>::to_vec))?;
                let flag = worker.cancellation().clone();
                let jacobian = crate::workflow::dynamics::analysis("diagnostic.original_jacobian", || worker.jacobian(&candidate))?;
                let entries = (0..jacobian.ncols())
                    .flat_map(|j| {
                        jacobian
                            .row_idx_of_col(j)
                            .zip(jacobian.val_of_col(j))
                            .map(move |(i, v)| (i, j, *v))
                    })
                    .map(|(i, j, v)| (i, j, v * scaled_rows[i] / scaled_columns[j]))
                    .collect::<Vec<_>>();
                let matrix = crate::workflow::dynamics::analysis("diagnostic.full_scaled_svd", || {
                    pse_math::diagnostics::analyze_matrix::<GlobalRow, GlobalCol>(
                        jacobian.as_ref(), &scaled_rows, &scaled_columns, matrix_policy, &flag)
                });
                // A square Jacobian also gets FERAL's sparse 1-norm condition estimate in the
                // same scaling, which needs no dense budget (L-N6).
                let condition = (jacobian.nrows() == jacobian.ncols()).then(|| {
                    let nominals = scaled_columns.iter().map(|s| 1. / s).collect::<Vec<_>>();
                    crate::workflow::dynamics::analysis("diagnostic.sparse_condition_estimate", || {
                        if flag.load(std::sync::atomic::Ordering::Acquire) {
                            return Err(pse_backend_native::ProblemError::Cancelled);
                        }
                        let result = pse_backend_native::conditioning::jacobian_condition(
                            jacobian.as_ref(), &scaled_rows, &nominals)?;
                        if flag.load(std::sync::atomic::Ordering::Acquire) {
                            return Err(pse_backend_native::ProblemError::Cancelled);
                        }
                        Ok(result)
                    })
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
                                FindingKind::EquationLargeResidual,
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
                                FindingKind::JacobianExtremeEntry,
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
                            (&matrix.row_norms, &row_ids, FindingKind::JacobianExtremeRow),
                            (
                                &matrix.column_norms,
                                &columns,
                                FindingKind::JacobianExtremeColumn,
                            ),
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
                                    FindingKind::JacobianParallelRows,
                                    [rows[p.first], rows[p.second]],
                                    p.cosine,
                                )
                            })
                            .chain(matrix.parallel_columns.iter().map(|p| {
                                (
                                    FindingKind::JacobianParallelColumns,
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
                                    FindingKind::JacobianNumericalRankDeficiency,
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
                            pse_diagnostics::DiagnosticStage::ModelingDiagnostics,
                            [],
                            pse_diagnostics::DiagnosticRule::JacobianAnalysisInconclusive,
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
                            FindingKind::JacobianConditionEstimate,
                            locations(),
                            Some(estimate),
                            None,
                        ),
                        maximum,
                        product,
                    ),
                    Some(Ok(None)) => {
                        let mut f = finding(
                            FindingKind::JacobianConditionEstimate,
                            locations(),
                            None,
                            None,
                        );
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
                            pse_diagnostics::DiagnosticStage::ModelingDiagnostics,
                            [],
                            pse_diagnostics::DiagnosticRule::JacobianAnalysisInconclusive,
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
                use pse_model::diagnostic::DiagnosticProjection;
                // Preserve the producer's typed numerical cause and original occurrence roles.
                // A message-only envelope cannot support source-aware investigation/reduction.
                report.push(error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::ModelingDiagnostics), maximum, product);
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
            .observe_registered(services.preparation(),
                prepared.model.clone(),
                terms,
                values,
                compiler,
                providers,
                operation.driver(),
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
                    .modeling_term_diagnostics(magnitudes.clone(), policy.terms, operation)
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
                                FindingKind::EquationMismatchedTerm,
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
                                FindingKind::EquationCancelingTerms,
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
                let mut f = finding(FindingKind::EquationTermEvaluationFailed, [], None, None);
                f.observations
                    .insert("reason".into(), Observation::Text(error.to_string()));
                report.push(f, maximum, product);
            }
        }
        operation.check()?;
        Ok(report)

        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_compiler::workspace::ModelingCaseBindings;
    use pse_kernels::DerivativeOrder;
    #[tokio::test]
    async fn kernel_diagnostics_retain_requested_structure_without_objective_or_solve_admission() {
        use pse_backend_native::{
            routing::{AssessmentState, Ineligible},
            solve::{Backend, SolveIntent, SolverSelection},
        };
        let rt = super::super::super::tests::runtime();
        let services = rt.execution_services();
        let declarations = pse_authoring::language::parse(
            "package p { def D {var x:Scalar; eq lo:x*x>=4; eq hi:x*x<=1; eq spare:x*x<=100; annotation start x(1.5);} }",
            SemanticId::from_bytes([175; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|declaration| declaration.name == "D")
            .unwrap()
            .declaration_id;
        let package = rt
            .modeling_package(declarations, super::super::super::tests::physical())
            .await
            .unwrap();
        let analysis = ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Bindings::default(),
            limits: Limits::default(),
            case: Default::default(),
            order: DerivativeOrder::First,
            compiler: super::super::super::tests::compiler_profile(),
            solver: crate::math::solves::SolverProfile {
                intent: SolveIntent::Optimize,
                selection: SolverSelection::Explicit(Backend::Ipopt),
                ..super::super::super::tests::profile()
            },
            numerical: Default::default(),
        };
        let cancel = crate::CancelSource::new();
        let prepared = package
            .prepare_diagnostics(services.preparation(), &analysis, &cancel)
            .await
            .unwrap();
        let decision = &prepared.route_decision;
        assert_eq!(decision.state, AssessmentState::Refused);
        assert!(decision.selected.is_none());
        assert!(decision.refusal.is_some());
        let candidate = decision
            .eligibility
            .iter()
            .find(|candidate| candidate.backend == Backend::Ipopt)
            .unwrap();
        assert!(candidate.reasons.contains(&Ineligible::NoObjective));
        let assessment = decision.structure.as_ref().unwrap();
        assert_eq!(
            assessment.row(decision.snapshot, 0),
            candidate
                .structure
                .as_ref()
                .unwrap()
                .row(decision.snapshot, 0)
        );
        assert_eq!(assessment.variables.len(), 1);
        assert_eq!(assessment.equations.len(), 3);
        assert_eq!(
            assessment.witness.matching,
            prepared.model.case.structure().matching
        );
        assert_eq!(
            assessment.witness.allocation_identity(),
            prepared
                .model
                .case
                .structural_witness()
                .allocation_identity()
        );
        assert!(prepared.model.case.compiled().coefficients.is_none());
        let values = prepared.model.values.clone();
        let policy: ModelingDiagnosticPolicy =
            <ModelingDiagnosticPolicy as crate::boundary_documents::BoundaryDocument>::WIRE
                .decode_slice(include_bytes!(
                    "../../../../../packages/reference/diagnostics/idaes-2.13.json"
                ))
                .unwrap();
        let origin = crate::workflow::DiagnosticOrigin {
            run_id: pse_operations::mint_id(),
            attempt_id: None,
            step: Some(0),
            native_attempt: Some(0),
            operation: None,
            failure: Some(BoundaryDiagnostic::new(
                BoundaryClass::InvalidModel,
                pse_diagnostics::DiagnosticStage::ModelingDiagnostics,
                [],
                pse_diagnostics::DiagnosticRule::CandidateEvaluationFailed,
            )),
        };
        let operation = crate::workflow::DiagnosticOperation::new(
            &cancel,
            std::time::Duration::from_secs(60),
            Some(origin.clone()),
        )
        .unwrap();
        let report = package
            .diagnose_case(
                &services,
                prepared.clone(),
                values.clone(),
                policy.clone(),
                analysis.compiler,
                &operation,
            )
            .await
            .unwrap();
        assert!(report.complete, "{:?}", report.findings);
        assert_eq!(report.origin.as_ref().unwrap().run_id, origin.run_id);
        let table = report.table("runtime.modeling_diagnostics").unwrap();
        let raw = table
            .batch()
            .column_by_name("origin_json")
            .unwrap()
            .as_any()
            .downcast_ref::<datafusion::arrow::array::StringArray>()
            .unwrap()
            .value(0);
        let readback: crate::workflow::DiagnosticOrigin = serde_json::from_str(raw).unwrap();
        assert_eq!(readback.run_id, origin.run_id);
        assert_eq!(readback.failure.unwrap().rule, origin.failure.unwrap().rule);
        let expired = crate::workflow::DiagnosticOperation::new(
            &cancel,
            std::time::Duration::from_nanos(1),
            None,
        )
        .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        assert!(
            package
                .diagnose_case(
                    &services,
                    prepared,
                    values,
                    policy,
                    analysis.compiler,
                    &expired
                )
                .await
                .is_err()
        );
        assert_eq!(report.statistics["free_variables"], 1);
        assert_eq!(report.statistics["equalities"], 0);
        assert_eq!(report.statistics["inequalities"], 3);
        assert_eq!(report.route_decision.state, AssessmentState::Refused);
        assert!(report.route_decision.selected.is_none());
        assert!(
            package
                .prepare_analysis_attempt(
                    services.preparation(),
                    &analysis,
                    Default::default(),
                    &cancel
                )
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn kernel_diagnostics_missing_starts_remain_missing_and_solves_stay_strict() {
        let rt = super::super::super::tests::runtime();
        let services = rt.execution_services();
        let physical = super::super::super::tests::physical();
        let declarations = pse_authoring::language::parse(
            "package p { def Root {var x:Scalar; eq e:x==2; annotation bounds x(0,1);} }",
            SemanticId::from_bytes([175; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(declarations, physical).await.unwrap();
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
            .prepare_diagnostics(services.preparation(), &analysis, &cancel)
            .await
            .unwrap();
        assert!(prepared.model.values.scalars.is_empty());
        let key = prepared.model.case.compiled().plan.structure().key();
        let policy: ModelingDiagnosticPolicy =
            <ModelingDiagnosticPolicy as crate::boundary_documents::BoundaryDocument>::WIRE
                .decode_slice(include_bytes!(
                    "../../../../../packages/reference/diagnostics/idaes-2.13.json"
                ))
                .unwrap();
        let report = package
            .diagnose_case(
                &services,
                prepared.clone(),
                prepared.model.values.clone(),
                policy.clone(),
                analysis.compiler,
                &crate::workflow::DiagnosticOperation::new(
                    &cancel,
                    std::time::Duration::from_secs(60),
                    None,
                )
                .unwrap(),
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
                .any(|f| f.rule == pse_diagnostics::DiagnosticRule::VariableMissingValue)
        );
        assert!(
            package
                .prepare_analysis_attempt(
                    services.preparation(),
                    &analysis,
                    Default::default(),
                    &cancel
                )
                .await
                .is_err()
        );
        analysis.case.values.insert("x".into(), 2.);
        let prepared = package
            .prepare_diagnostics(services.preparation(), &analysis, &cancel)
            .await
            .unwrap();
        assert_eq!(key, prepared.model.case.compiled().plan.structure().key());
        let report = package
            .diagnose_case(
                &services,
                prepared.clone(),
                prepared.model.values.clone(),
                policy,
                analysis.compiler,
                &crate::workflow::DiagnosticOperation::new(
                    &cancel,
                    std::time::Duration::from_secs(60),
                    None,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.rule == pse_diagnostics::DiagnosticRule::VariableOutsideUpperBound)
        );
        assert!(
            package
                .prepare_analysis_attempt(
                    services.preparation(),
                    &analysis,
                    Default::default(),
                    &cancel
                )
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
                .prepare_diagnostics(services.preparation(), &analysis, &cancel)
                .await
                .unwrap_err()
                .to_string()
                .contains("missing frozen")
        );
    }
    #[tokio::test]
    async fn kernel_diagnostics_name_sources_and_keep_numerical_rank_distinct_from_structure() {
        let rt = super::super::super::tests::runtime();
        let services = rt.execution_services();
        let physical = super::super::super::tests::physical();
        let rows = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; var y:Scalar; eq a:x+y==0; eq b:2*x+2*y==0; } }",
            SemanticId::from_bytes([83; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical).await.unwrap();
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
            .prepare_diagnostics(services.preparation(), &analysis, &cancel)
            .await
            .unwrap();
        assert_eq!(prepared.model.case.structure().matching.len(), 2);
        let source_map = rt
            .modeling_diagnostic_source_map(
                &prepared,
                1 << 20,
                std::time::Duration::from_secs(10),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(
            source_map.document().source_identity,
            prepared.model.case.compiled().plan.structure().key()
        );
        assert_eq!(
            source_map.document().numerical_identity,
            prepared.numerics.key
        );
        assert_eq!(
            source_map
                .document()
                .row_nominals
                .iter()
                .map(|(row, _)| *row)
                .collect::<Vec<_>>(),
            source_map.document().rows
        );
        assert_eq!(
            source_map
                .document()
                .variable_nominals
                .iter()
                .map(|(column, _)| *column)
                .collect::<Vec<_>>(),
            source_map.document().columns
        );
        assert!(!source_map.document().templates.is_empty());
        assert!(!source_map.document().instances.is_empty());
        assert!(
            source_map
                .document()
                .fields
                .iter()
                .all(|field| field.document == Some(SemanticId::from_bytes([83; 16])))
        );
        assert!(
            source_map
                .document()
                .templates
                .iter()
                .all(|template| !template.arithmetic.is_empty())
        );
        assert!(
            source_map
                .document()
                .templates
                .iter()
                .flat_map(|template| &template.arithmetic)
                .all(|origin| origin.source.is_some() == origin.source_unavailable.is_none())
        );
        for instance in &source_map.document().instances {
            assert_eq!(
                instance.contributions.len(),
                prepared.model.case.compiled().plan.structure().instances()[instance.ordinal]
                    .contributions
                    .len()
            );
            assert!(!instance.demands.is_empty());
            assert!(instance.demands.iter().all(|demand| {
                source_map
                    .document()
                    .templates
                    .iter()
                    .any(|template| template.template == demand.template)
            }));
        }
        assert!(
            source_map
                .document()
                .native_pullback_unavailable
                .contains("no transformed native attempt")
        );
        assert!(
            rt.modeling_diagnostic_source_map(
                &prepared,
                1,
                std::time::Duration::from_secs(10),
                &cancel
            )
            .await
            .is_err()
        );
        let stopped_map = crate::CancelSource::new();
        stopped_map.cancel();
        assert!(
            rt.modeling_diagnostic_source_map(
                &prepared,
                1 << 20,
                std::time::Duration::from_secs(10),
                &stopped_map
            )
            .await
            .is_err()
        );
        let mut attributed = prepared.model.model.compiled().clone();
        let mut model = (*attributed.model).clone();
        let original = model.equations.iter().map(|row| row.id).collect::<Vec<_>>();
        let variable = model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(".x"))
            .unwrap();
        let variable_id = variable.id;
        let alias_id = pse_ids::named_id(variable_id, "diagnostic-alias");
        let mut alias_lineage = variable.lineage.clone();
        alias_lineage.path.push_str(".supplied_alias");
        model.ports.insert(
            alias_id,
            pse_modeling::specialize::Port {
                id: alias_id,
                symbol: variable_id,
                lineage: alias_lineage,
            },
        );
        attributed.model = Arc::new(model).into();
        let mut combined = finding(
            FindingKind::JacobianNumericalRankDeficiency,
            original.iter().copied().chain([variable_id]),
            None,
            None,
        );
        let missing = pse_ids::named_id(root.as_id(), "truly-unmapped");
        combined.causes.push(finding(
            FindingKind::EquationLargeResidual,
            [original[0], missing],
            None,
            None,
        ));
        attribute(&mut combined, &attributed);
        assert!(original.iter().all(|source| {
            combined
                .locations
                .iter()
                .any(|location| location.source == *source)
        }));
        assert!(combined.sources.contains(&alias_id));
        assert!(
            combined
                .locations
                .iter()
                .any(|location| location.source == alias_id
                    && location.path.ends_with(".supplied_alias"))
        );
        for source in &original {
            assert!(
                combined
                    .locations
                    .iter()
                    .any(|location| location.source == *source
                        && location.document == Some(SemanticId::from_bytes([83; 16]))
                        && location.occurrence_role.is_some()
                        && location.occurrence_position.is_some())
            );
        }
        // A plain declaration has authored lineage but no expression occurrence.
        // Neither it nor the supplied alias may invent a field or byte range.
        for source in [variable_id, alias_id] {
            assert!(
                combined
                    .locations
                    .iter()
                    .any(|location| location.source == source
                        && location.document.is_none()
                        && location.occurrence_role.is_none()
                        && location.occurrence_position.is_none()
                        && location.start.is_none()
                        && location.end.is_none())
            );
        }
        assert!(
            combined.causes[0]
                .locations
                .iter()
                .any(|location| location.source == original[0])
        );
        assert!(
            !combined.causes[0]
                .locations
                .iter()
                .any(|location| location.source == missing)
        );
        let locations = combined.locations.len();
        attribute(&mut combined, &attributed);
        assert_eq!(combined.locations.len(), locations);
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
                &services,
                prepared.clone(),
                values.clone(),
                p.clone(),
                compiler,
                &crate::workflow::DiagnosticOperation::new(
                    &cancel,
                    std::time::Duration::from_secs(60),
                    None,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(report.matrix.as_ref().unwrap().rank, 1);
        for mode in &report.matrix.as_ref().unwrap().modes {
            assert!(
                mode.left_residual
                    .is_none_or(|value| value <= mode.residual_bound)
            );
            assert!(
                mode.right_residual
                    .is_none_or(|value| value <= mode.residual_bound)
            );
            assert!(mode.normalization_error < 1e-12);
        }
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
                &services,
                prepared.clone(),
                points.clone(),
                p.clone(),
                compiler,
                3,
                &crate::workflow::DiagnosticOperation::new(
                    &cancel,
                    std::time::Duration::from_secs(60),
                    None,
                )
                .unwrap(),
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
                &services,
                prepared.clone(),
                points.clone(),
                p.clone(),
                compiler,
                1,
                &crate::workflow::DiagnosticOperation::new(
                    &cancel,
                    std::time::Duration::from_secs(60),
                    None,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(samples.stop, DiagnosticSampleStop::SampleLimit);
        assert_eq!(samples.unattempted, 2);
        let stopped = crate::CancelSource::new();
        stopped.cancel();
        let samples = package
            .diagnose_samples(
                &services,
                prepared.clone(),
                points,
                p.clone(),
                compiler,
                3,
                &crate::workflow::DiagnosticOperation::new(
                    &stopped,
                    std::time::Duration::from_secs(60),
                    None,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(samples.stop, DiagnosticSampleStop::Cancelled);
        assert_eq!(samples.unattempted, 3);
        assert!(report.findings.iter().any(|f| f.rule
            == pse_diagnostics::DiagnosticRule::JacobianParallelRows
            && f.locations.len() == 2));
        // The square Jacobian [[1, 1], [2, 2]] is singular for the sparse LU as well.
        let condition = report
            .findings
            .iter()
            .find(|f| f.rule == pse_diagnostics::DiagnosticRule::JacobianConditionEstimate)
            .unwrap();
        assert!(!condition.observations.contains_key("value"));
        assert!(condition.observations.contains_key("reason"));
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.rule == pse_diagnostics::DiagnosticRule::EquationCancelingTerms)
        );
        assert!(!report.findings.iter().any(|f| matches!(
            f.rule,
            pse_diagnostics::DiagnosticRule::StructuralUnderdetermined
                | pse_diagnostics::DiagnosticRule::StructuralOverdetermined
        )));
        #[cfg(feature = "solver-kinsol")]
        {
            let physical = super::super::super::tests::physical();
            let rows = pse_authoring::language::parse("package p { def Root { var x:Scalar; var unused:Scalar; implicit i {var y:Scalar; eq e:y==2; annotation start y(1);} realize r on i using nested; eq e:x==i.y; } }", SemanticId::from_bytes([175; 16]), pse_authoring::language::IdentityPolicy::Named, pse_authoring::ParseBudget::default()).unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let nested = rt.modeling_package(rows, physical).await.unwrap();
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
                .prepare_diagnostics(services.preparation(), &analysis, &cancel)
                .await
                .unwrap();
            assert_eq!(prepared.providers.len(), 1);
            let values = prepared.model.values.clone();
            let report = nested
                .diagnose_case(
                    &services,
                    prepared,
                    values,
                    p,
                    compiler,
                    &crate::workflow::DiagnosticOperation::new(
                        &cancel,
                        std::time::Duration::from_secs(60),
                        None,
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            assert!(report.complete, "{:?}", report.findings);
            // The complete original rows y - 2 and x - y are independent, while
            // the unused column still makes the structure underdetermined.
            assert_eq!(report.matrix.as_ref().unwrap().rank, 2);
            assert!(
                report
                    .findings
                    .iter()
                    .any(|f| f.rule == pse_diagnostics::DiagnosticRule::StructuralUnderdetermined)
            );
            assert_eq!(report.statistics["unused_variables"], 1);
            assert!(
                report
                    .findings
                    .iter()
                    .any(|f| f.rule == pse_diagnostics::DiagnosticRule::VariableUnused)
            );
        }
        #[cfg(feature = "solver-highs")]
        {
            let native = package
                .diagnose_jacobian_optimization(
                    &services,
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
                    &crate::workflow::DiagnosticOperation::new(
                        &cancel,
                        std::time::Duration::from_secs(60),
                        None,
                    )
                    .unwrap(),
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
    /// Retained original invocation or failure, independent of scientific permission.
    pub origin: Option<super::super::DiagnosticOrigin>,
    /// Identity of this analysis run.
    pub run_id: RunId,
    /// Certificates, degenerate sets and every native attempt.
    pub evidence: pse_backend_native::jacobian_diagnostics::Report,
    pub(in crate::workflow::modeling) quantities: Arc<pse_quantity::QuantityRegistry>,
    pub(in crate::workflow::modeling) services: crate::workflow::PreparationServices,
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
        services: &crate::workflow::ExecutionServices,
        prepared: ModelingDiagnosticPreparation,
        values: CaseValues,
        policy: pse_backend_native::jacobian_diagnostics::Policy,
        controls: pse_backend_native::solve::Controls,
        operation: &super::super::DiagnosticOperation,
    ) -> Result<ModelingJacobianOptimization, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let activity = services.preparation().admit_activity()?;
        activity
            .scope(async {
                controls
                    .validate()
                    .map_err(crate::math::MathRuntimeError::from)?;
                operation.check()?;
                let scope = operation.scope().clone();
                let control = scope.cancellation_control().clone();
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
                let normalization = pse_math::normalization::Normalization::from_policy(
                    &numerics,
                    plan.columns(),
                    &rows,
                )
                .map_err(crate::math::MathRuntimeError::from)?;
                let source_identity = plan.structure().key();
                let numerical_identity = numerics.key;
                let allowance = controls
                    .report_allowance()
                    .map_err(crate::math::MathRuntimeError::from)?
                    .checked_mul(policy.maximum_attempts)
                    .and_then(|n| n.checked_add(policy.maximum_entries.checked_mul(128)?))
                    .and_then(|n| n.checked_add(values.scalars.len().checked_mul(128)?))
                    .and_then(|n| n.checked_add(operation.retained_bytes()))
                    .ok_or_else(|| contract("native diagnostic extent"))?;
                let service = services.preparation().math();
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
                let assembly = crate::math::MathService::within_task(
                    &scope,
                    operation.driver(),
                    service.assemble(prepared.case),
                )
                .await?;
                let (evidence, owner) = service
                    .with_owned_worker(
                        assembly,
                        providers,
                        operation.driver(),
                        Some((scope.clone(), control)),
                        move |mut worker| {
                            let execution = pse_backend_native::solve::Execution::within(
                                scope.cancellation_control().clone(),
                                &controls,
                                scope,
                            )?;
                            let original = worker.jacobian(&values)?;
                            execution.check()?;
                            let mut normalized = original.clone();
                            for j in 0..normalized.ncols() {
                                let indices = normalized.row_idx_of_col(j).collect::<Vec<_>>();
                                for (i, v) in indices.into_iter().zip(normalized.val_of_col_mut(j))
                                {
                                    *v = *v * normalization.variables[j] / normalization.rows[i];
                                }
                            }
                            let evidence = pse_backend_native::jacobian_diagnostics::analyze(
                                normalized.as_ref(),
                                &rows,
                                policy,
                                &controls,
                                execution,
                            )?;
                            Ok((evidence, retained))
                        },
                    )
                    .await?;
                operation.check()?;
                Ok(ModelingJacobianOptimization {
                    services: services.preparation().clone(),
                    quantities: self.physical.quantities.clone(),
                    origin: operation.origin.clone(),
                    run_id: pse_operations::mint_id(),

                    source_identity,
                    numerical_identity,
                    point,
                    row_nominals,
                    variable_nominals,
                    policy,
                    evidence,
                    _owner: owner,
                })
            })
            .await
    }
}
