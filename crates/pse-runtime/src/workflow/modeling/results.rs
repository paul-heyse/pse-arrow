// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Knowledge checks supplement original-space solver qualification without rewriting termination.
use super::cases::ModelingSolvePreparation;
use super::*;
use crate::math::solves::Outcome;
use pse_compiler::workspace::{ModelingHint, ModelingOutput, ObjectiveBound, Profile};
use pse_math::binding::CaseValues;
use pse_model::generated::identities::RunId;
use pse_modeling::annotation::AnnotationValue;
use std::{collections::BTreeSet, sync::Arc};

use pse_model::generated::enums::{
    ExtrapolationPolicy, ModelingCheckBasis as Basis, ModelingCheckKind as CheckKind,
};
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
    pub run_id: RunId,
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
fn stamp_start(outcome: &mut Outcome, run: RunId, attempt: usize) {
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
        // A least-infeasible stop names the original rows it leaves violated (ADR-0109
        // item 3) beside the unsatisfied knowledge checks.
        let violated = match &self.outcome {
            Outcome::Native(native) => native
                .least_infeasible
                .as_ref()
                .map(|label| label.violated.iter().map(|v| v.id).collect::<Vec<_>>())
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let mut result = D::new(
            C::TrialRejected,
            "modeling-result",
            self.checks
                .iter()
                .filter(|c| !c.satisfied)
                .map(|c| c.source_id.as_id())
                .chain(violated),
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
            // Registry names, never Rust `Debug` output (F30).
            result.observations.insert(
                "termination".into(),
                Observation::Text(native.termination.category.as_str().into()),
            );
            result.observations.insert(
                "qualification".into(),
                Observation::Text(native.qualification.as_str().into()),
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
        run_id: RunId,
        attempt: usize,
        mut outcome: Outcome,
        point: assessment::AssessedPoint,
        native_owner: Arc<pse_columnar::AllocationLease>,
    ) -> Self {
        let completion = crate::workflow::numerics::complete(
            outcome.candidate_use(),
            &point.checks,
            point.complete && point.error.is_none(),
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
}
/// A step's certified dual bound on its authored objective (ADR-0119 Outcome 5): a bound
/// the backend established over the declared box, in the objective's orientation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::workflow) struct CertifiedBound {
    /// The dual bound, in the objective's canonical units.
    pub value: f64,
    /// The authored orientation it bounds: a lower bound when minimizing.
    pub sense: pse_math::binding::ObjectiveSense,
}
impl CertifiedBound {
    /// The certified bound of a step, present only when its assurance is `global_bound` or
    /// `exact_certificate` and the native dual bound is finite. A check never starts a solve
    /// to obtain one.
    pub(in crate::workflow) fn of(outcome: &Outcome) -> Option<Self> {
        use pse_backend_native::solve::Assurance;
        let Outcome::Native(native) = outcome else {
            return None;
        };
        if !matches!(
            native.termination.assurance,
            Assurance::GlobalBound | Assurance::ExactCertificate
        ) {
            return None;
        }
        let global = native.evidence.global.as_ref()?;
        Some(Self {
            value: global.dual_bound.filter(|v| v.is_finite())?,
            sense: global.sense,
        })
    }
    /// Whether this bound can establish `check`: it bounds the objective on the side the
    /// check constrains.
    fn establishes(self, check: ObjectiveBound) -> bool {
        use pse_math::binding::ObjectiveSense as Sense;
        matches!(
            (check, self.sense),
            (ObjectiveBound::Lower { .. }, Sense::Minimize)
                | (ObjectiveBound::Upper { .. }, Sense::Maximize)
        )
    }
}
/// One `annotation check` at a step: at the point, or, for an objective-bound check with a
/// certified bound, against that bound. Returns the indicator value and its basis.
fn check_value(
    point: f64,
    objective_bound: Option<(ObjectiveBound, f64)>,
    certified: Option<CertifiedBound>,
) -> (f64, Basis) {
    match (objective_bound, certified) {
        (Some((check, compared)), Some(bound)) if bound.establishes(check) => (
            if check.holds(bound.value, compared) {
                1.
            } else {
                0.
            },
            Basis::GlobalBound,
        ),
        _ => (point, Basis::Point),
    }
}
/// Shared demand and interpretation for steady candidates and trajectory samples.
pub(in crate::workflow) type AssessmentScope = BTreeSet<(SemanticId, DeclarationId)>;
/// Each obligation is indivisible even when its value, bounds or tolerance depend
/// on different inputs. Consumers may postpone a whole obligation, never one term.
pub(in crate::workflow) fn assessment_units(
    product: &pse_compiler::workspace::PreparedModeling,
) -> BTreeMap<(SemanticId, DeclarationId), BTreeSet<SemanticId>> {
    let mut units = BTreeMap::<_, BTreeSet<_>>::new();
    for output in &product.admitted.outputs {
        let key = match output {
            ModelingOutput::Test { id, .. } => {
                Some((*id, product.model.expectations[id].lineage.declaration))
            }
            ModelingOutput::Hint {
                target,
                declaration,
                kind:
                    ModelingHint::Check
                    | ModelingHint::ObjectiveBound(_)
                    | ModelingHint::ValidLower
                    | ModelingHint::ValidUpper,
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
/// Assess the observed point. An objective-bound check reads `certified`, the step's
/// certified dual bound, when the step has one (ADR-0119 Outcome 5).
#[expect(
    clippy::too_many_arguments,
    reason = "assessment reads the run, product, values and observations under the numerical policy, registry, report selection, scope and the step's certified bound"
)]
pub(in crate::workflow) fn assess_observations(
    run_id: RunId,
    product: &pse_compiler::workspace::PreparedModeling,
    values: &CaseValues,
    observed: &BTreeMap<SemanticId, f64>,
    numerics: &pse_model::numerics::ResolvedNumericalPolicy,
    quantities: &pse_quantity::QuantityRegistry,
    include_reports: bool,
    scope: Option<&AssessmentScope>,
    certified: Option<CertifiedBound>,
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
        .assess_expectations_for(observed, &expected)
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
            basis: Basis::Point,
        })
        .collect::<Vec<_>>();
    // The compiler's typed classification of objective-bound checks, with the compared
    // side observed at this point (it depends on no decision).
    let objective_bounds = product
        .admitted
        .outputs
        .iter()
        .filter_map(|output| match output {
            ModelingOutput::Hint {
                target,
                declaration,
                kind: ModelingHint::ObjectiveBound(check),
            } if selected(*target, *declaration) => Some(
                observed
                    .get(&output.row_id())
                    .map(|compared| ((*target, *declaration), (*check, *compared)))
                    .ok_or_else(|| contract("objective-bound check side absent")),
            ),
            _ => None,
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut magnitudes = BTreeMap::new();
    for output in &product.admitted.outputs {
        match output {
            ModelingOutput::Hint {
                target,
                declaration,
                kind: ModelingHint::Check,
            } if selected(*target, *declaration) => {
                let (value, basis) = check_value(
                    observed[&output.row_id()],
                    objective_bounds.get(&(*target, *declaration)).copied(),
                    certified,
                );
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
                    basis,
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
                    basis: Basis::Point,
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
                basis: Basis::Point,
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
                let extrapolate = *policy == ExtrapolationPolicy::Extrapolate;
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
                    satisfied: inside || extrapolate,
                    within_validity: Some(inside),
                    extrapolation_allowed: Some(extrapolate),
                    basis: Basis::Point,
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
        // The failure first, then the informational bound tightenings (ADR-0103 item 4).
        let tightenings = self
            .prepared
            .model
            .tightenings
            .iter()
            .map(pse_modeling::DomainTightening::boundary_diagnostic);
        for (ordinal, finding) in self.diagnostic().into_iter().chain(tightenings).enumerate() {
            columns
                .push(analysis_tables::finding_row(
                    self.run_id,
                    ordinal as i64,
                    &finding,
                ))
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
                pse_modeling::specialize::root_instance(root),
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
    /// A native step with the given assurance and, when present, a global dual bound on a
    /// minimized objective: a synthetic stand-in for a certifying solve.
    fn step(assurance: pse_backend_native::solve::Assurance, dual_bound: Option<f64>) -> Outcome {
        use pse_backend_native::{self as native, solve::*};
        let id = SemanticId::from_bytes([1; 16]);
        let contract = native::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![native::Variable {
                id,
                lower: -2.0,
                upper: 2.0,
            }],
            rows: vec![],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let mut report = SolveReport::new(
            Backend::Scip,
            &contract,
            NativeTermination {
                code: 0,
                name: "fixture".into(),
                message: None,
                category: Termination::Success,
                assurance,
            },
            &Execution::new(Arc::default(), &Controls::default()),
        );
        report.evidence.global = Some(GlobalEvidence {
            fidelity: pse_math::factorable::Fidelity::Exact,
            domain: pse_ids::ContentHash::from_bytes([2; 32]),
            sense: pse_math::binding::ObjectiveSense::Minimize,
            feasibility: 1e-9,
            gap_relative: 1e-6,
            gap_absolute: 1e-9,
            dual_bound,
            primal_bound: Some(0.0),
            gap: Some(0.0),
            nodes: 1,
            readback: true,
            dual: BoundSource::ExactExport,
            primal: PrimalSource::Backend,
            infeasible: false,
            exact: assurance == Assurance::ExactCertificate,
        });
        Outcome::Native(Box::new(report))
    }
    #[tokio::test]
    async fn objective_bound_check_uses_certified_bound() {
        use pse_backend_native::solve::Assurance;
        let runtime = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let names = BTreeMap::from([(
            "Scalar".into(),
            physical.quantities.neutral_dimensionless().unwrap(),
        )]);
        // A tangent-plane-style stability check: the minimized objective stays above -c.
        let source = "package p { def Root { param c: Scalar = 0.5; var x: Scalar; let f: Scalar = x*x; eq g: x <= 2; annotation bounds x(-2, 2); annotation start x(1); annotation objective f(minimize); annotation check f(f > -c); } }";
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
        let cancel = crate::CancelSource::new();
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                pse_kernels::DerivativeOrder::First,
                super::super::super::tests::compiler_profile(),
                crate::math::solves::SolverProfile {
                    intent: pse_backend_native::solve::SolveIntent::Optimize,
                    ..super::super::super::tests::profile()
                },
                crate::math::solves::NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let product = prepared.model.model.compiled();
        let row = |wanted: fn(&ModelingHint) -> bool| {
            product
                .admitted
                .outputs
                .iter()
                .find(|o| matches!(o, ModelingOutput::Hint { kind, .. } if wanted(kind)))
                .map(ModelingOutput::row_id)
                .unwrap()
        };
        let check = row(|k| *k == ModelingHint::Check);
        let side = row(|k| matches!(k, ModelingHint::ObjectiveBound(_)));
        // The point indicator and the compared side, as the step's observation program
        // would report them.
        let assess = |point: f64, outcome: &Outcome| {
            let observed = BTreeMap::from([(check, point), (side, -0.5)]);
            let (checks, _) = assess_observations(
                RunId::from_bytes([7; 16]),
                product,
                &prepared.model.values,
                &observed,
                prepared.solve.numerics(),
                &prepared.source.quantities,
                false,
                None,
                CertifiedBound::of(outcome),
            )
            .unwrap();
            let [check] = checks.as_slice() else {
                panic!("one check expected, got {checks:?}");
            };
            (check.satisfied, check.value, check.basis)
        };
        // A certified dual bound below -c: the global minimum may violate the check, so it
        // fails although the local point satisfies it.
        assert_eq!(
            assess(1.0, &step(Assurance::GlobalBound, Some(-0.75))),
            (false, 0.0, Basis::GlobalBound)
        );
        // A certified bound above -c establishes it over the box, whatever the point says.
        assert_eq!(
            assess(0.0, &step(Assurance::GlobalBound, Some(1e-9))),
            (true, 1.0, Basis::GlobalBound)
        );
        assert_eq!(
            assess(0.0, &step(Assurance::ExactCertificate, Some(0.0))),
            (true, 1.0, Basis::GlobalBound)
        );
        // Without a certified bound the check is evaluated at the point, and says so.
        for outcome in [
            step(Assurance::LocalStationary, Some(1e-9)),
            step(Assurance::NativeOptimal, Some(1e-9)),
            step(Assurance::GlobalBound, None),
            step(Assurance::GlobalBound, Some(f64::NEG_INFINITY)),
        ] {
            assert_eq!(assess(1.0, &outcome), (true, 1.0, Basis::Point));
            assert_eq!(assess(0.0, &outcome), (false, 0.0, Basis::Point));
        }
        // The bound establishes only a check on the side it bounds.
        let lower = ObjectiveBound::Lower { strict: true };
        let upper = ObjectiveBound::Upper { strict: true };
        let minimum = CertifiedBound {
            value: 1.0,
            sense: pse_math::binding::ObjectiveSense::Minimize,
        };
        assert_eq!(
            check_value(0.0, Some((lower, 0.5)), Some(minimum)),
            (1.0, Basis::GlobalBound)
        );
        assert_eq!(
            check_value(0.0, Some((upper, 2.0)), Some(minimum)),
            (0.0, Basis::Point)
        );
        assert_eq!(check_value(1.0, None, Some(minimum)), (1.0, Basis::Point));
    }
}
