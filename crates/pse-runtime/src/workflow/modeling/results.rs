// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Knowledge checks supplement original-space solver qualification without rewriting termination.
use super::cases::ModelingSolvePreparation;
use super::*;
use crate::math::solves::{Outcome, SolveSequence};
use pse_compiler::workspace::{ModelingHint, ModelingOutput, Profile};
use pse_math::binding::CaseValues;
use pse_modeling::annotation::AnnotationValue;
use std::{collections::BTreeSet, sync::Arc};

use pse_model::generated::enums::ModelingCheckKind as CheckKind;
/// Registry-owned check and report rows are also the public Rust values.
pub use pse_model::generated::runtime::modeling_checks::Row as ModelingCheck;
pub use pse_model::generated::runtime::modeling_reports::Row as ModelingReport;
/// Completed solve and explicit model-level qualification. Diagnostic evidence never
/// changes the native report or turns a failed local solve into infeasibility proof.
#[derive(Clone, Debug)]
pub struct ModelingResult(Arc<ModelingResultData>);
/// Immutable result storage shared with every retained public handle.
#[derive(Debug)]
pub struct ModelingResultData {
    pub outcome: Outcome,
    pub values: CaseValues,
    pub checks: Vec<ModelingCheck>,
    pub reports: Vec<ModelingReport>,
    pub run_id: SemanticId,
    runtime: Runtime,
    /// Projection of `completion`; never decided separately.
    pub accepted: bool,
    /// The §16.6 candidate-use completion shared with the published assessment.
    pub(in crate::workflow) completion: crate::workflow::numerics::Completed,
    pub validation_error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    pub prepared: ModelingSolvePreparation,
    _owner: Arc<pse_columnar::AllocationLease>,
    _native_owner: Arc<pse_columnar::AllocationLease>,
}
impl std::ops::Deref for ModelingResult {
    type Target = ModelingResultData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
fn stamp_start(outcome: &mut Outcome, run: SemanticId, attempt: usize) {
    if let Outcome::Native(native) = outcome {
        if let Some(seed) = &mut native.warm_start {
            seed.origin = Some(pse_backend_native::solve::SeedOrigin {
                run: Some(run),
                attempt,
            });
        }
        if let Some(receipt) = &mut native.start_receipt
            && let Some(previous) = receipt.previous_attempt
            && let Some(seed) = &mut receipt.seed
        {
            seed.origin = Some(pse_backend_native::solve::SeedOrigin {
                run: Some(run),
                attempt: previous,
            });
        }
    }
}
impl ModelingResult {
    /// Structured reason this original-model result was not accepted.
    pub fn diagnostic(&self) -> Option<pse_model::diagnostic::BoundaryDiagnostic> {
        use pse_model::diagnostic::{BoundaryClass as C, BoundaryDiagnostic as D, Observation};
        if self.accepted {
            return None;
        }
        if let Outcome::Native(native) = &self.outcome
            && let Some(cause) = native.callback_failure()
        {
            return Some(super::super::diagnostics::observed(
                cause,
                "native-callback",
            ));
        }
        if let Outcome::Native(native) = &self.outcome
            && let Some(cause) = native.validation_failure()
        {
            return Some(super::super::diagnostics::observed(
                cause,
                "native-validation",
            ));
        }
        if let Some(error) = &self.validation_error {
            return Some(error.clone());
        }
        if let Outcome::Rejected(error) = &self.outcome {
            return Some(
                WorkflowError::Math(crate::math::MathRuntimeError::Shared(error.clone()))
                    .boundary_diagnostic(),
            );
        }
        let mut result = D::new(
            C::TrialRejected,
            "modeling-result",
            self.checks
                .iter()
                .filter(|c| !c.satisfied)
                .map(|c| c.source_id),
            "modeling.qualification.rejected",
        );
        if let Outcome::Native(native) = &self.outcome {
            use pse_backend_native::solve::Termination as T;
            result.class = match native.termination.category {
                T::Cancelled => C::Cancelled,
                T::TimeLimit | T::IterationLimit => C::ResourceLimit,
                T::Numerical => C::Numerical,
                T::Inconclusive => C::Inconclusive,
                _ => C::TrialRejected,
            };
            result.observations.insert(
                "termination".into(),
                Observation::Text(format!("{:?}", native.termination.category)),
            );
            result.observations.insert(
                "qualification".into(),
                Observation::Text(format!("{:?}", native.qualification)),
            );
            result.observations.insert(
                "candidate_use".into(),
                Observation::Text(self.completion.decision.usability.as_str().into()),
            );
            result.observations.insert(
                "candidate_reason".into(),
                Observation::Text(self.completion.decision.reason.as_str().into()),
            );
        }
        Some(result)
    }
}
pub(in crate::workflow) fn result_bytes(
    prepared: &ModelingSolvePreparation,
) -> Result<usize, WorkflowError> {
    let report_strings = prepared
        .model
        .model
        .compiled()
        .model
        .annotations
        .iter()
        .try_fold(0usize, |n, a| {
            let label = if let AnnotationValue::Report(label) = &a.value {
                label.len()
            } else {
                0
            };
            n.checked_add(label.checked_add(a.lineage.path.len())?.checked_mul(2)?)
        })
        .ok_or_else(|| contract("model report string extent"))?;
    let bytes = prepared
        .model
        .values
        .scalars
        .len()
        .checked_mul(128)
        .and_then(|n| n.checked_add(report_strings))
        .and_then(|n| n.checked_add(prepared.model.model.compiled().model.annotations.len() * 256))
        .and_then(|n| n.checked_add(prepared.model.model.compiled().admitted.outputs.len() * 128))
        .ok_or_else(|| contract("modeling result extent"))?;
    Ok(bytes)
}
impl ModelingResult {
    pub(in crate::workflow) fn from_assessment(
        prepared: ModelingSolvePreparation,
        run_id: SemanticId,
        attempt: usize,
        mut outcome: Outcome,
        point: sequence::AssessedPoint,
        native_owner: Arc<pse_columnar::AllocationLease>,
    ) -> Self {
        let completion = crate::workflow::numerics::complete(
            outcome.candidate_use(),
            &point.checks,
            point.error.is_none(),
            prepared.solve.numerics().policy.closure,
        );
        stamp_start(&mut outcome, run_id, attempt);
        Self(Arc::new(ModelingResultData {
            runtime: prepared.source.runtime.clone(),
            prepared,
            run_id,
            outcome,
            values: point.values,
            checks: point.checks,
            reports: point.reports,
            accepted: completion.permits_use(),
            completion,
            validation_error: point.error,
            _owner: point.owner,
            _native_owner: native_owner,
        }))
    }
}
impl ModelingPackage {
    /// Execute one prepared case, join native destruction, then check knowledge obligations.
    pub async fn solve_case(
        &self,
        prepared: ModelingSolvePreparation,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        let mut prepared = prepared;
        prepared.compiler = compiler;
        let handle = prepared.start()?;
        let wait = handle.wait();
        tokio::pin!(wait);
        let joined = tokio::select! {
            result = &mut wait => result?,
            () = cancel.cancelled() => {handle.cancel();wait.await?}
        };
        match &joined.report {
            Ok(super::super::RunReport::Modeling(results)) => results
                .first()
                .cloned()
                .ok_or_else(|| contract("authored solve completed without a result")),
            Err(error) => Err(WorkflowError::Shared(error.clone())),
            _ => Err(contract("authored solve report mismatch")),
        }
    }
    /// Intermediate specifications retain their model checks; final-fixture oracles
    /// apply only to the separate original-specification solve.
    pub(in crate::workflow) async fn solve_initialization_trial(
        &self,
        prepared: ModelingSolvePreparation,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        self.solve_assessed(prepared, compiler, false, cancel).await
    }
    async fn solve_assessed(
        &self,
        prepared: ModelingSolvePreparation,
        compiler: Profile,
        include_expectations: bool,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        let handle = self.runtime.shared.math().solve(SolveSequence {
            steps: vec![prepared.solve.clone()],
            continue_independent: false,
            result_limit: 1,
        })?;
        let control = handle.cancellation();
        let finish = handle.finish();
        tokio::pin!(finish);
        let sequence = tokio::select! {
            result=&mut finish=>result?,
            ()=cancel.cancelled()=>{control.cancel();finish.await?}
        };
        let (mut outcomes, _, native_owner) = sequence.into_parts();
        let outcome = outcomes
            .pop()
            .ok_or_else(|| contract("solve completed without an outcome"))?;
        self.finish_assessed(
            prepared,
            compiler,
            include_expectations,
            pse_authoring::ids::uuid_v7(),
            outcome,
            native_owner,
            cancel,
        )
        .await
    }
    pub(in crate::workflow) async fn finish_assessed(
        &self,
        prepared: ModelingSolvePreparation,
        compiler: Profile,
        include_expectations: bool,
        run_id: SemanticId,
        outcome: Outcome,
        native_owner: Arc<pse_columnar::AllocationLease>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        let mut values = prepared.model.values.clone();
        if let Outcome::Native(r) = &outcome
            && let Some(candidate) = &r.candidate
        {
            if r.variables.len() != candidate.primal.len() {
                return Err(contract("native candidate coordinate extent"));
            }
            for (id, v) in r.variables.iter().zip(&candidate.primal) {
                values.scalars.insert(*id, *v);
            }
        }
        let native = outcome.candidate_use();
        let policy = prepared.solve.numerics().policy.closure;
        let bytes = result_bytes(&prepared)?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("modeling:qualified-result", bytes)?;
        let mut result = ModelingResultData {
            run_id,
            runtime: self.runtime.clone(),
            outcome,
            values,
            checks: vec![],
            reports: vec![],
            accepted: false,
            completion: crate::workflow::numerics::complete(native, &[], false, policy),
            validation_error: None,
            prepared,
            _owner: owner,
            _native_owner: native_owner,
        };
        let has_candidate = matches!(&result.outcome, Outcome::Constant(_))
            || matches!(&result.outcome,Outcome::Native(r) if r.candidate.is_some());
        if has_candidate {
            match self
                .qualify(
                    result.run_id,
                    &result.prepared,
                    &result.values,
                    compiler,
                    include_expectations,
                    cancel,
                )
                .await
            {
                Ok((checks, reports)) => {
                    result.checks = checks;
                    result.reports = reports;
                }
                Err(error) => result.validation_error = Some(error.boundary_diagnostic()),
            }
        }
        result.completion = crate::workflow::numerics::complete(
            native,
            &result.checks,
            has_candidate && result.validation_error.is_none(),
            policy,
        );
        result.accepted = result.completion.permits_use();
        stamp_start(&mut result.outcome, run_id, 0);
        Ok(ModelingResult(Arc::new(result)))
    }
    async fn qualify(
        &self,
        run_id: SemanticId,
        prepared: &ModelingSolvePreparation,
        values: &CaseValues,
        compiler: Profile,
        include_expectations: bool,
        cancel: &crate::CancelSource,
    ) -> Result<(Vec<ModelingCheck>, Vec<ModelingReport>), WorkflowError> {
        let product = prepared.model.model.compiled();
        let mut units = assessment_units(product);
        if !include_expectations {
            for expectation in product.model.expectations.values() {
                units.remove(&(expectation.id, expectation.lineage.declaration));
            }
        }
        let scope = units.keys().copied().collect();
        let rows = units.into_values().flatten().collect();
        let observed = self
            .observe_registered(
                prepared.model.model.clone(),
                rows,
                values.clone(),
                compiler,
                prepared.providers.clone(),
                cancel,
            )
            .await?;
        assess_observations(
            run_id,
            product,
            values,
            &observed,
            prepared.solve.numerics(),
            &self.quantities,
            true,
            Some(&scope),
        )
    }
}
/// Shared demand and interpretation for steady candidates and trajectory samples.
pub(in crate::workflow) type AssessmentScope = BTreeSet<(SemanticId, SemanticId)>;
/// Each obligation is indivisible even when its value, bounds or tolerance depend
/// on different inputs. Consumers may postpone a whole obligation, never one term.
pub(in crate::workflow) fn assessment_units(
    product: &pse_compiler::workspace::PreparedModeling,
) -> BTreeMap<(SemanticId, SemanticId), BTreeSet<SemanticId>> {
    let mut units = BTreeMap::<_, BTreeSet<_>>::new();
    for output in &product.admitted.outputs {
        let key = match output {
            ModelingOutput::Test { id, .. } => {
                Some((*id, product.model.expectations[id].lineage.declaration))
            }
            ModelingOutput::Hint {
                target,
                declaration,
                kind: ModelingHint::Check | ModelingHint::ValidLower | ModelingHint::ValidUpper,
            } => Some((*target, *declaration)),
            ModelingOutput::OriginalEquation(id) => {
                Some((*id, product.model.elastic[id].original.lineage.declaration))
            }
            ModelingOutput::Contribution { accumulator, .. } => Some((
                *accumulator,
                product.model.closures[accumulator].lineage.declaration,
            )),
            _ => None,
        };
        if let Some(key) = key {
            units.entry(key).or_default().insert(output.row_id());
        }
    }
    for a in &product.model.annotations {
        if matches!(
            a.value,
            AnnotationValue::Report(_) | AnnotationValue::Valid { .. }
        ) {
            units
                .entry((a.target, a.lineage.declaration))
                .or_default()
                .insert(ModelingOutput::Member(a.target).row_id());
        }
    }
    units
}
pub(in crate::workflow) fn observation_rows(
    product: &pse_compiler::workspace::PreparedModeling,
    _values: &CaseValues,
) -> Result<BTreeSet<SemanticId>, WorkflowError> {
    Ok(assessment_units(product).into_values().flatten().collect())
}
pub(in crate::workflow) fn assess_observations(
    run_id: SemanticId,
    product: &pse_compiler::workspace::PreparedModeling,
    values: &CaseValues,
    observed: &BTreeMap<SemanticId, f64>,
    numerics: &pse_model::numerics::ResolvedNumericalPolicy,
    quantities: &pse_quantity::QuantityRegistry,
    include_reports: bool,
    scope: Option<&AssessmentScope>,
) -> Result<(Vec<ModelingCheck>, Vec<ModelingReport>), WorkflowError> {
    let selected = |target, source| scope.is_none_or(|ids| ids.contains(&(target, source)));
    let expected = product
        .model
        .expectations
        .values()
        .filter(|t| selected(t.id, t.lineage.declaration))
        .map(|t| t.id)
        .collect();
    let mut checks = product
        .assess_expectations_for(&observed, &expected)
        .map_err(|e| contract(e.to_string()))?
        .into_iter()
        .map(|t| ModelingCheck {
            step: 0,
            run_id,
            sample_index: 0,
            time: None,
            target_id: t.id,
            source_id: t.declaration,
            kind: CheckKind::Expectation,
            value: (t.actual - t.expected).abs(),
            tolerance: Some(t.tolerance),
            satisfied: t.passed,
            within_validity: None,
            extrapolation_allowed: None,
        })
        .collect::<Vec<_>>();
    let mut magnitudes = BTreeMap::new();
    for output in &product.admitted.outputs {
        match output {
            ModelingOutput::Hint {
                target,
                declaration,
                kind: ModelingHint::Check,
            } if selected(*target, *declaration) => {
                let value = observed[&output.row_id()];
                checks.push(ModelingCheck {
                    step: 0,
                    run_id,
                    sample_index: 0,
                    time: None,
                    target_id: *target,
                    source_id: *declaration,
                    kind: CheckKind::Check,
                    value,
                    tolerance: None,
                    satisfied: value == 1.,
                    within_validity: None,
                    extrapolation_allowed: None,
                });
            }
            ModelingOutput::Contribution {
                accumulator,
                contribution,
            } if selected(
                *accumulator,
                product.model.closures[accumulator].lineage.declaration,
            ) =>
            {
                magnitudes.insert(*contribution, observed[&output.row_id()]);
            }
            ModelingOutput::OriginalEquation(id)
                if selected(*id, product.model.elastic[id].original.lineage.declaration) =>
            {
                let original = &product.model.elastic[id].original;
                let pse_authoring::dsl::EquationKind::Relation { sense, .. } =
                    original.equation.kind
                else {
                    return Err(contract("original elastic equation shape"));
                };
                let value = observed[&output.row_id()];
                let residual = match sense {
                    pse_authoring::dsl::EquationSense::Eq => value.abs(),
                    pse_authoring::dsl::EquationSense::Le => value.max(0.),
                    pse_authoring::dsl::EquationSense::Ge => (-value).max(0.),
                };
                let tolerance = numerics
                    .targets
                    .iter()
                    .find(|t| {
                        t.id == *id && t.kind == pse_model::generated::enums::NumericalTarget::Row
                    })
                    .ok_or_else(|| contract("original row budget absent"))?
                    .budget;
                checks.push(ModelingCheck {
                    step: 0,
                    run_id,
                    sample_index: 0,
                    time: None,
                    target_id: *id,
                    source_id: original.lineage.declaration,
                    kind: CheckKind::OriginalEquation,
                    value: residual,
                    tolerance: Some(tolerance),
                    satisfied: residual <= tolerance,
                    within_validity: None,
                    extrapolation_allowed: None,
                });
            }
            _ => {}
        }
    }
    let closures = product
        .model
        .closures
        .values()
        .filter(|c| selected(c.id, c.lineage.declaration))
        .map(|c| c.id)
        .collect();
    for closure in product
        .model
        .assess_closures(&magnitudes, &closures)
        .map_err(|e| contract(e.to_string()))?
    {
        if let Some(satisfied) = closure.satisfied {
            checks.push(ModelingCheck {
                step: 0,
                run_id,
                sample_index: 0,
                time: None,
                target_id: closure.accumulator,
                source_id: product.model.closures[&closure.accumulator]
                    .lineage
                    .declaration,
                kind: CheckKind::Closure,
                value: closure.net,
                tolerance: Some(closure.tolerance),
                satisfied,
                within_validity: None,
                extrapolation_allowed: None,
            });
        }
    }
    let value = |target: SemanticId| -> Result<f64, WorkflowError> {
        values
            .scalars
            .get(&target)
            .copied()
            .or_else(|| {
                product
                    .admitted
                    .outputs
                    .iter()
                    .find(|o| matches!(o,ModelingOutput::Member(id) if *id==target))
                    .and_then(|o| observed.get(&o.row_id()).copied())
            })
            .ok_or_else(|| contract("model check/report target absent"))
    };
    let mut reports = Vec::new();
    let mut reported = BTreeSet::new();
    for a in &product.model.annotations {
        if !selected(a.target, a.lineage.declaration) {
            continue;
        }
        match &a.value {
            AnnotationValue::Report(label) if include_reports => {
                if !reported.insert((a.target, a.lineage.declaration)) {
                    return Err(contract("duplicate report source/target"));
                }
                let symbol = product
                    .model
                    .symbols
                    .get(&a.target)
                    .ok_or_else(|| contract("reported target is not a scalar member"))?;
                let pse_modeling::Type::Quantity(scheme) = &symbol.ty else {
                    return Err(contract("reported target has no physical contract"));
                };
                let quantity = scheme
                    .resolve(quantities, &BTreeMap::new())
                    .map_err(|e| contract(e.to_string()))?;
                let unit = quantities
                    .quantity_type(quantity)
                    .map_err(|e| contract(e.to_string()))?
                    .canonical_unit;
                reports.push(ModelingReport {
                    step: 0,
                    run_id,
                    target_id: a.target,
                    source_id: a.lineage.declaration,
                    label: label.clone(),
                    path: symbol.lineage.path.clone(),
                    quantity_id: quantity.as_id(),
                    unit_id: unit.as_id(),
                    value: value(a.target)?,
                });
            }
            AnnotationValue::Valid { policy, .. } => {
                let endpoint = |kind| -> Result<f64, WorkflowError> {
                    product.admitted.outputs.iter().find(|o|matches!(o,ModelingOutput::Hint{target,declaration,kind:k} if *target==a.target && *declaration==a.lineage.declaration && *k==kind)).and_then(|o|observed.get(&o.row_id()).copied()).ok_or_else(||contract("validity endpoint absent"))
                };
                let lo = endpoint(ModelingHint::ValidLower)?;
                let hi = endpoint(ModelingHint::ValidUpper)?;
                let v = value(a.target)?;
                if lo > hi {
                    return Err(contract("reversed validity interval"));
                }
                let inside = v >= lo && v <= hi;
                if policy != "reject" && policy != "extrapolate" {
                    return Err(contract(
                        "validity policy must be reject or explicitly selected extrapolate",
                    ));
                }
                checks.push(ModelingCheck {
                    step: 0,
                    run_id,
                    sample_index: 0,
                    time: None,
                    target_id: a.target,
                    source_id: a.lineage.declaration,
                    kind: CheckKind::Validity,
                    value: v,
                    tolerance: None,
                    satisfied: inside || policy == "extrapolate",
                    within_validity: Some(inside),
                    extrapolation_allowed: Some(policy == "extrapolate"),
                });
            }
            _ => {}
        }
    }
    Ok((checks, reports))
}

impl ModelingResult {
    /// Encode generated contracts with the shared reserve-before-growth columnar owner.
    /// Returned columns may outlive the model, result and source runtime handle.
    pub fn tables(
        &self,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        use pse_model::HeapUsage;
        let pool = self.runtime.shared.pool();
        let cancel = pse_columnar::CancellationToken::new();
        let scratch = self.runtime.shared.math().reserve(
            "modeling:result-row-copy",
            self.checks
                .iter()
                .map(HeapUsage::owned_bytes)
                .chain(self.reports.iter().map(HeapUsage::owned_bytes))
                .max()
                .unwrap_or(0),
        )?;
        let mut columns =
            pse_relations::columnar::Collection::new(&self.runtime.registry, &pool, &cancel);
        columns.ensure::<ModelingCheck>().map_err(relation)?;
        columns.ensure::<ModelingReport>().map_err(relation)?;
        columns
            .ensure::<pse_model::generated::runtime::modeling_findings::Row>()
            .map_err(relation)?;
        if let Some(failure) = self.diagnostic() {
            columns
                .push(analysis_tables::finding_row(self.run_id, 0, &failure))
                .map_err(relation)?;
        }
        for row in &self.checks {
            columns.push(row.clone()).map_err(relation)?;
        }
        for row in &self.reports {
            columns.push(row.clone()).map_err(relation)?;
        }
        let result = columns.finish().map_err(relation)?;
        drop(scratch);
        Ok(result
            .into_values()
            .map(|batch| (batch.relation_id(), batch))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_compiler::workspace::{ModelingCaseBindings, ModelingVariableState};
    use pse_relations::columnar::RelationRow;
    #[tokio::test]
    async fn kernel_indexed_reports_and_extrapolation_have_generated_owned_transports() {
        let runtime = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let neutral = physical.quantities.neutral_dimensionless().unwrap();
        let unit = physical
            .quantities
            .quantity_type(neutral)
            .unwrap()
            .canonical_unit;
        let names = BTreeMap::from([("Scalar".into(), neutral)]);
        let source = "package p { entity kind Item {} entity Item a {} entity Item b {} set items: Set<Item> = {a,b}; def Root { var x[j in items]: Scalar; eq e[j in items]: x[j] == 2; annotation start x(2); annotation report x(\"value\"); annotation valid x(0,1,extrapolate); } }";
        let rows = pse_authoring::language::parse(
            source,
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
        let package = runtime.modeling_package(rows, physical, names).unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let case = ModelingCaseBindings {
            values: BTreeMap::new(),
            variables: ["x[a]", "x[b]"]
                .into_iter()
                .map(|p| {
                    (
                        p.into(),
                        ModelingVariableState {
                            fixed: Some(true),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        };
        let prepared = package
            .prepare_solve(
                root,
                root,
                Bindings::default(),
                Limits::default(),
                case,
                pse_kernels::DerivativeOrder::First,
                compiler,
                super::super::super::tests::profile(),
                crate::math::solves::NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let result = package
            .solve_case(prepared, compiler, &cancel)
            .await
            .unwrap();
        assert!(result.accepted, "{result:?}");
        let before = runtime.shared.pool().reserved();
        let retained = result.clone();
        assert!(std::ptr::eq(&result.values, &retained.values));
        assert!(std::ptr::eq(&result.outcome, &retained.outcome));
        assert_eq!(runtime.shared.pool().reserved(), before);
        drop(retained);
        assert_eq!(result.reports.len(), 2);
        assert!(result.reports.iter().all(|r| r.label == "value"
            && r.value == 2.
            && r.quantity_id == neutral.as_id()
            && r.unit_id == unit.as_id()));
        assert_ne!(result.reports[0].target_id, result.reports[1].target_id);
        assert_eq!(result.checks.len(), 2);
        assert!(result.checks.iter().all(|c| c.kind == CheckKind::Validity
            && c.within_validity == Some(false)
            && c.extrapolation_allowed == Some(true)
            && c.satisfied));
        let mut tables = result.tables().unwrap();
        let reports = tables
            .remove(&pse_relations::generated::runtime::modeling_reports::RELATION_ID)
            .unwrap();
        let checks = tables
            .remove(&pse_relations::generated::runtime::modeling_checks::RELATION_ID)
            .unwrap();
        let expected = result.reports.clone();
        drop(result);
        drop(package);
        drop(runtime);
        assert_eq!(ModelingReport::rows(&reports).unwrap(), expected);
        assert!(
            ModelingCheck::rows(&checks)
                .unwrap()
                .iter()
                .all(|c| c.within_validity == Some(false))
        );
    }
}
