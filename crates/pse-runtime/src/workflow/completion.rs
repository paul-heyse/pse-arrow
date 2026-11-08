// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Completion owns qualification and lineage. Exporters only copy this product.
use super::{RunResult, WorkflowError};
use crate::math::solves::Outcome;
use pse_ids::{ContentHash, FramedHasher};
use pse_model::generated::{
    enums::*,
    runtime::{computation_runs, run_lineage, solve_runs},
};
mod member;
pub use member::MemberScientificPermission;

/// Immutable semantic projection shared by native readers, Arrow and publication.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Completion {
    /// Standalone completion document interpretation.
    pub version: pse_model::document::Version<3>,
    /// Whole original request decision; absence never authorizes scientific reuse.
    pub permission: Option<ScientificPermission>,
    /// Independently requested original modeling steps, never whole-request promotion.
    pub step_permissions: Vec<ScientificPermission>,
    /// Original independently assessed members. Historical absence is unavailable.
    #[serde(default)]
    pub member_permissions: Vec<MemberScientificPermission>,
    /// Producer-scoped mathematical validity composed with its governing original decision.
    pub products: Vec<ProductPermission>,
    /// Final physical closure and usability decisions, never recomputed by an exporter.
    pub assessments: Vec<pse_model::generated::runtime::candidate_assessments::Row>,
    /// Retained engineering outcomes; historical no-goal products have an empty collection.
    #[serde(default)]
    pub accuracy_goals: Vec<pse_model::engineering_accuracy::GoalAssessment>,
    /// Source-attributed final errors and unavailable fit diagnostics.
    pub diagnostics: Vec<pse_model::diagnostic::BoundaryDiagnostic>,
    /// Algebraic outcomes, including requested steps that were not attempted.
    pub solves: Vec<solve_runs::Row>,
    /// Dynamic or fitting outcome, with candidate evidence separate from qualification.
    pub computation: Option<computation_runs::Row>,
    /// Complete request lineage, independent of this execution's unique identity.
    pub lineage: Vec<run_lineage::Row>,
}

pse_contract::projection_struct! {
/// One completion-owned original scope, with evidence and reasons retained together.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScientificPermission {
    projection: { name: "ScientificPermissionSnapshot", rust_path: crate::workflow::ScientificPermission,
        version: 1, direction: pse_contract::BoundaryDirection::Output, construction: ObservationOnly,
        reserved_tags: [], reserved_names: [] };
    /// Actual producer-owned original point, absent when no candidate was retained.
    pub reference_point: Option<pse_ids::SemanticId> => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Absent for the complete request; otherwise the original independent step.
    pub step: Option<i64> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Required original observations in this scope.
    pub requested: usize => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Actually retained original observations.
    pub observed: usize => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Whether the original requested report succeeded.
    pub report_succeeded: bool => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Result permission; seed evidence remains on the original assessments.
    pub decision: CandidateUse => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original refusal reasons, including incomplete coverage and report failure.
    pub refusals: Vec<CandidateRefusal> => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original applicability/incumbent qualifiers, without strengthening.
    pub qualifiers: Vec<CandidateQualifier> => { tag: 8, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
impl ScientificPermission {
    /// This exact scope supplies scientific result permission.
    pub fn permits_result(&self) -> bool {
        matches!(
            self.decision,
            CandidateUse::Usable | CandidateUse::QualifiedUnclosed
        )
    }
    pub(in crate::workflow) fn assess(
        step: Option<i64>,
        succeeded: bool,
        requested: usize,
        observed: usize,
        assessments: &[pse_model::generated::runtime::candidate_assessments::Row],
    ) -> Self {
        let mut refusals = assessments
            .iter()
            .flat_map(|a| a.refusals.iter().copied())
            .collect::<Vec<_>>();
        let qualifiers = assessments
            .iter()
            .flat_map(|a| a.qualifiers.iter().copied())
            .collect();
        if !succeeded {
            refusals.push(CandidateRefusal::NativeOutcome);
        }
        if requested == 0 || observed < requested {
            refusals.push(CandidateRefusal::CoverageUnavailable);
        }
        if assessments.is_empty() {
            refusals.push(CandidateRefusal::NoCandidate);
        }
        if assessments.iter().any(|a| !a.permits_result) && refusals.is_empty() {
            refusals.push(CandidateRefusal::Unqualified);
        }
        let permitted = succeeded
            && requested > 0
            && observed >= requested
            && !assessments.is_empty()
            && assessments.iter().all(|a| a.permits_result);
        Self {
            reference_point: None,
            step,
            requested,
            observed,
            report_succeeded: succeeded,
            decision: if !permitted {
                CandidateUse::Unusable
            } else if assessments
                .iter()
                .any(|a| a.usability == CandidateUse::QualifiedUnclosed)
            {
                CandidateUse::QualifiedUnclosed
            } else {
                CandidateUse::Usable
            },
            refusals,
            qualifiers,
        }
    }
}
pse_contract::projection_enum! {
/// Original semantic scope of a product, inside its enclosing run/attempt.
/// Transport ranges and scientific quantity names cannot substitute for this identity.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "scope", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProductScope {
    projection: { name: "ScientificProductScope", rust_path: crate::workflow::ProductScope,
        version: 2, direction: pse_contract::BoundaryDirection::Both,
        reserved_tags: [], reserved_names: [] };
    /// A whole task product, such as a fit's selected covariance.
    Task => 1;
    /// A product of one independently requested original step.
    Step {
        /// Original independent step, never a selected row ordinal.
        step: i64 => { tag: 1, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    } => 2;
    /// A product at an exact original point, optionally in an independent step/member.
    Point {
        /// Original independent step, or task scope when absent.
        step: Option<i64> => { tag: 1, direction: pse_contract::BoundaryDirection::Both, rules: [] };
        /// Original solution member when the producer supplies a member binding.
        member: Option<pse_ids::SemanticId> => { tag: 2, direction: pse_contract::BoundaryDirection::Both, rules: [] };
        /// Actual producer-owned retained point identity.
        point: pse_ids::SemanticId => { tag: 3, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    } => 3;
    /// A complete original member, not an arbitrary point or transport subset.
    Member {
        /// Exact originally declared instance inside the enclosing run and attempt.
        member: pse_model::generated::identities::InstanceId => { tag: 1, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    } => 4;
}
}
impl ProductScope {
    fn governing<'a>(&self, completion: &'a Completion) -> Option<&'a ScientificPermission> {
        let step = match self {
            Self::Task => return completion.permission.as_ref(),
            Self::Member { member } => {
                return completion
                    .member_permission(*member)
                    .and_then(MemberScientificPermission::native_permission);
            }
            Self::Step { step } => Some(*step),
            Self::Point {
                step,
                member,
                point,
            } => {
                if member.is_some() {
                    return None;
                }
                let permission = match step {
                    Some(step) if *step >= 0 => completion.original_step_permission(*step),
                    Some(_) => None,
                    None => completion.permission.as_ref(),
                }?;
                return (permission.reference_point == Some(*point)).then_some(permission);
            }
        };
        match step {
            Some(step) if step >= 0 => completion.original_step_permission(step),
            Some(_) => None,
            None => completion.permission.as_ref(),
        }
    }
}
pse_contract::projection_struct! {
/// Local validity remains meaningful even when original permission withholds a product.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProductPermission {
    projection: { name: "ScientificProductPermissionSnapshot", rust_path: crate::workflow::ProductPermission,
        version: 1, direction: pse_contract::BoundaryDirection::Output, construction: ObservationOnly,
        reserved_tags: [], reserved_names: [] };
    /// Exact original product scope; never inferred from quantity or transport layout.
    pub scope: ProductScope => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Existing registry-owned derived scientific quantity.
    pub quantity: DerivedQuantity => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// The independent mathematical assessment.
    pub validity: pse_model::generated::structures::LocalValidity => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Governing original and accepted derived-work limitations, without strengthening.
    pub qualifiers: Vec<CandidateQualifier> => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// The completed scientific product decision.
    pub decision: CandidateUse => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
impl ProductPermission {
    fn retain_qualifiers(&mut self, qualifiers: impl IntoIterator<Item = CandidateQualifier>) {
        for qualifier in qualifiers {
            if !self.qualifiers.contains(&qualifier) {
                self.qualifiers.push(qualifier);
            }
        }
    }
    /// Both the original governing request and local mathematics permit this product.
    pub fn permits_result(&self) -> bool {
        matches!(
            self.decision,
            CandidateUse::Usable | CandidateUse::QualifiedUnclosed
        )
    }
}
impl Completion {
    /// Select one exact original member; missing or ambiguous facts withhold use.
    pub fn member_permission(
        &self,
        member: pse_model::generated::identities::InstanceId,
    ) -> Option<&MemberScientificPermission> {
        let mut matches = self
            .member_permissions
            .iter()
            .filter(|permission| permission.member() == member);
        let result = matches.next()?;
        matches.next().is_none().then_some(result)
    }
    /// Whole-request result permission, unavailable when no original fact was captured.
    pub fn governing_permission(&self) -> Option<bool> {
        self.permission
            .as_ref()
            .map(ScientificPermission::permits_result)
    }
    /// Missing original permission is never success.
    pub fn permits_result(&self) -> bool {
        self.governing_permission() == Some(true)
    }
    /// Governing original step fact, or the aggregate scope when no independent steps were requested.
    pub fn step_permission(&self, step: usize) -> Option<&ScientificPermission> {
        if self.step_permissions.is_empty() {
            self.permission.as_ref()
        } else {
            i64::try_from(step)
                .ok()
                .and_then(|step| self.original_step_permission(step))
        }
    }
    /// Resolve one original independent step without interpreting collection order as authority.
    fn original_step_permission(&self, step: i64) -> Option<&ScientificPermission> {
        let mut matches = self
            .step_permissions
            .iter()
            .filter(|permission| permission.step == Some(step));
        let permission = matches.next()?;
        matches.next().is_none().then_some(permission)
    }
    /// The recorded product fact; consumers never reconstruct its conjunction.
    pub fn product_permission(
        &self,
        scope: &ProductScope,
        quantity: DerivedQuantity,
    ) -> Option<&ProductPermission> {
        let mut matches = self
            .products
            .iter()
            .filter(|p| &p.scope == scope && p.quantity == quantity);
        let fact = matches.next()?;
        // Ambiguous facts never select an arbitrary first product.
        matches.next().is_none().then_some(fact)
    }
    /// Compose a later derived owner's actual mathematical assessment with original permission.
    pub fn derive_permission(
        &self,
        scope: ProductScope,
        quantity: DerivedQuantity,
        validity: pse_model::generated::structures::LocalValidity,
    ) -> ProductPermission {
        // A step coordinate alone is diagnostic provenance, not a retained point.
        // Current task-derived products are the fit-owned estimate/statistical families.
        let scope_supports_quantity = match &scope {
            ProductScope::Step { .. } => false,
            ProductScope::Task => {
                matches!(
                    quantity,
                    DerivedQuantity::ParameterCovariance
                        | DerivedQuantity::WaldInterval
                        | DerivedQuantity::ProfileInterval
                ) || (quantity == DerivedQuantity::PropagatedCovariance
                    && self
                        .product_permission(
                            &ProductScope::Task,
                            DerivedQuantity::ParameterCovariance,
                        )
                        .is_some_and(ProductPermission::permits_result))
            }
            // Whole-member trajectory permission carries no retained mathematical
            // point or member-derived product assessment.
            ProductScope::Member { .. } => false,
            ProductScope::Point { .. } => true,
        };
        let governing = scope.governing(self);
        let decision = if validity.certified && scope_supports_quantity {
            governing
                .filter(|p| p.permits_result())
                .map_or(CandidateUse::Unusable, |p| p.decision)
        } else {
            CandidateUse::Unusable
        };
        ProductPermission {
            qualifiers: governing.map_or_else(Vec::new, |permission| permission.qualifiers.clone()),
            scope,
            quantity,
            validity,
            decision,
        }
    }
}

impl RunResult {
    fn capture_product_permissions(&self, completion: &mut Completion) {
        if let super::RunRequest::Modeling(requests) = self.original_request() {
            for (ordinal, request) in requests.iter().enumerate() {
                self.capture_modeling_product_permissions(
                    completion,
                    request,
                    self.modeling_result(ordinal),
                    self.modeling_step(ordinal) as i64,
                );
            }
        } else if let Ok(super::RunReport::Initialization(report)) = self.report() {
            for (ordinal, attempt) in report.attempts.iter().enumerate() {
                if let Ok(result) = &attempt.result {
                    self.capture_modeling_product_permissions(
                        completion,
                        &result.prepared,
                        Some(result),
                        ordinal as i64,
                    );
                }
            }
        }
        let super::RunOutcome::Fit {
            request,
            result: Ok(report),
        } = &self.outcome
        else {
            return;
        };
        let point = report
            .solve
            .as_ref()
            .and_then(|s| s.evidence.local.as_ref())
            .and_then(|p| p.as_ref().ok());
        let mut add = |quantity,
                       outcome: Result<(), (WithheldReason, String)>,
                       point,
                       qualifiers: &[CandidateQualifier]| {
            let validity = super::local_analysis::record(
                outcome.as_ref().map_err(|(r, s)| (*r, s.clone())),
                point,
                false,
            );
            let mut fact = completion.derive_permission(ProductScope::Task, quantity, validity);
            fact.retain_qualifiers(qualifiers.iter().copied());
            completion.products.push(fact);
        };
        if let Some(covariance) = &report.covariance {
            add(
                DerivedQuantity::ParameterCovariance,
                covariance
                    .values
                    .as_ref()
                    .map(|_| ())
                    .map_err(|e| (e.reason(), e.to_string())),
                (covariance.approximation == CovarianceApproximation::Exact)
                    .then_some(point)
                    .flatten(),
                &[],
            );
        }
        if let Some(wald) = &report.wald {
            add(
                DerivedQuantity::WaldInterval,
                wald.as_ref()
                    .map(|_| ())
                    .map_err(|e| (e.reason(), e.to_string())),
                None,
                &[],
            );
        }
        if let Some(profiles) = &report.profiles {
            let qualifiers = profiles.as_ref().map_or_else(
                |_| Vec::new(),
                |chains| {
                    chains
                        .iter()
                        .flat_map(|chain| chain.qualifiers.iter().copied())
                        .collect::<Vec<_>>()
                },
            );
            add(
                DerivedQuantity::ProfileInterval,
                profiles
                    .as_ref()
                    .map_err(|e| (e.reason(), e.to_string()))
                    .and_then(|chains| request.profile_interval_validity(chains)),
                None,
                &qualifiers,
            );
        }
    }
    fn capture_modeling_product_permissions(
        &self,
        completion: &mut Completion,
        request: &super::ModelingSolvePreparation,
        result: Option<&super::ModelingResult>,
        step: i64,
    ) {
        let Some(sensitivity) = &request.profile.sensitivity else {
            return;
        };
        let native = result.and_then(|result| match &result.outcome {
            Outcome::Native(report) => Some(report.as_ref()),
            _ => None,
        });
        let input = super::local_analysis::Step {
            run_id: self.run_id,
            step,
            request: sensitivity,
            report: native,
            candidate: native.is_some_and(|report| report.candidate.is_some()),
            structure: request.model.case.compiled().plan.structure(),
            quantities: &request.source.physical.quantities,
        };
        let scope =
            result
                .filter(|_| input.candidate)
                .map_or(ProductScope::Step { step }, |result| ProductScope::Point {
                    step: Some(step),
                    member: None,
                    point: result.run_id.as_id(),
                });
        for (quantity, validity) in super::local_analysis::original_product_validities(&input) {
            completion.products.push(completion.derive_permission(
                scope.clone(),
                quantity,
                validity,
            ));
        }
    }
    /// The joined projection; observing it performs no mathematical work.
    pub fn completion(&self) -> Result<&Completion, &WorkflowError> {
        self.completion.as_ref().map_err(AsRef::as_ref)
    }

    fn capture_modeling_completion(
        &self,
        product: &mut Completion,
        requests: &[super::ModelingSolvePreparation],
        environment: ContentHash,
    ) -> Result<(), WorkflowError> {
        for (ordinal, request) in requests.iter().enumerate() {
            let step = self.modeling_step(ordinal) as i64;
            if let Some(result) = self.modeling_result(ordinal) {
                product.accuracy_goals.extend(
                    result
                        .completion
                        .accuracy
                        .iter()
                        .map(|g| g.row(self.run_id, step)),
                );
            } else {
                product.accuracy_goals.extend(
                    request
                        .solve
                        .numerics()
                        .policy
                        .goals
                        .iter()
                        .cloned()
                        .map(|goal| {
                            pse_math::engineering_accuracy::GoalResult::unavailable(
                                goal,
                                AccuracyUnavailableReason::MissingEvidence,
                            )
                            .row(self.run_id, step)
                        }),
                );
            }
            let solved = request.model.model.solved();
            let declaration = request.model.case.compiled().plan.structure();
            let outcome = self.modeling_result(ordinal).map(|result| &result.outcome);
            let native = match outcome {
                Some(Outcome::Native(r)) => Some(r.as_ref()),
                _ => None,
            };
            let constant = match outcome {
                Some(Outcome::Constant(r)) => Some(r.as_ref()),
                _ => None,
            };
            let quality = native
                .and_then(|r| r.quality.as_ref())
                .or_else(|| constant.map(|r| &r.quality));
            let observation = native
                .and_then(|r| r.observation.as_ref())
                .or_else(|| constant.map(|r| &r.observation));
            let candidate = native.and_then(|r| r.candidate.as_ref());
            let error = match (self.modeling_error(ordinal), outcome) {
                (Some(e), _) => Some(e.to_string()),
                (_, Some(Outcome::Rejected(e))) => Some(e.to_string()),
                _ => None,
            };
            product.solves.push(solve_runs::Row {
                run_id: self.run_id,
                step,
                model_id: Some(solved.model()),
                revision: Some(request.source.revision.identity().as_id()),
                case_id: solved.case(),
                instance_id: Some(solved.instance()),
                backend: native.map(|r| r.backend),
                native_code: native.map(|r| r.termination.code),
                native_status: native.map(|r| r.termination.name.clone()),
                state: if native.is_some() {
                    NativeRunState::Native
                } else if error.is_some() {
                    NativeRunState::Rejected
                } else if constant.is_some() {
                    NativeRunState::ConstantEvaluation
                } else {
                    NativeRunState::Unattempted
                },
                termination: native.map(|r| r.termination.category),
                assurance: native.map_or(NativeAssurance::None, |r| r.termination.assurance),
                qualification: native.map_or_else(
                    || {
                        if quality.is_some_and(|q| q.feasible()) {
                            NativeQualification::Feasible
                        } else {
                            NativeQualification::Unqualified
                        }
                    },
                    |r| r.qualification,
                ),
                candidate_kind: candidate
                    .map(|c| c.kind)
                    .or_else(|| constant.map(|_| NativeCandidateKind::ConstantEvaluation)),
                feasible: quality.map(|q| q.feasible()),
                objective: observation
                    .and_then(|o| o.objective)
                    .or_else(|| candidate.and_then(|c| c.objective)),
                objective_sense: declaration.objective().map(|o| match o.sense {
                    pse_math::binding::ObjectiveSense::Minimize => NativeObjectiveSense::Minimize,
                    pse_math::binding::ObjectiveSense::Maximize => NativeObjectiveSense::Maximize,
                }),
                objective_quantity_id: declaration.objective().map(|o| o.quantity.as_id()),
                validation_error: native
                    .and_then(|r| r.validation_failure().map(ToString::to_string)),
                error,
                transformation: native
                    .and_then(|r| r.preprocessing.as_ref().map(|p| p.transformation)),
                commitment: candidate.and_then(|c| c.commitment.as_ref()).map(|c| {
                    c.columns
                        .iter()
                        .map(|(id, (lower, upper))| {
                            solve_runs::RuntimeSolveRunsFieldCommitmentItem {
                                source_id: *id,
                                lower: *lower,
                                upper: *upper,
                            }
                        })
                        .collect()
                }),
            });

            let preparation = request
                .solve
                .preparation_identity()
                .map_err(crate::math::MathRuntimeError::from)?;
            let selected = request
                .solve
                .request_identity()
                .map_err(crate::math::MathRuntimeError::from)?;
            let profile = crate::math::solves::profile_key(&request.profile)
                .map_err(crate::math::MathRuntimeError::from)?;
            let mut identity = FramedHasher::new(pse_ids::Frame::CompletedRequestV2);
            identity
                .hash(&request.source.revision.identity().as_id())
                .id(&solved.instance().as_id())
                .hash(&preparation)
                .hash(&selected.as_id())
                .hash(&profile.as_id())
                .hash(&request.solve.numerics().key);
            // A result is traceable to what seeded it and to native state it reused:
            // the previous step's seed and the reuse of retained native state enter
            // its lineage identity (F25).
            frame_start(&mut identity, native);
            let mut actual_environment = FramedHasher::new(pse_ids::Frame::CompletedEnvironmentV1);
            actual_environment.hash(&environment).str(
                &self.services.outer_attestation().map_or_else(
                    || "local-unattested".into(),
                    |attestation| attestation.build.to_hex(),
                ),
            );
            if let Some(native) = native {
                actual_environment.str(native.backend.as_str());
                for (name, value) in &native.provenance {
                    actual_environment.str(name).str(value);
                }
            }
            let lineage = solved.lineage();
            product.lineage.push(run_lineage::Row {
                run_id: self.run_id,
                step,
                model_id: lineage.model_id,
                revision: request.source.revision.identity().as_id(),
                case_id: lineage.case_id,
                instance_id: lineage.instance_id,
                fit_id: lineage.fit_id,
                request_identity: identity.finish_hash(),
                preparation_identity: preparation,
                profile_identity: profile.as_id(),
                numerical_identity: request.solve.numerics().key,
                physical_identity: request.source.physical.key,
                environment_identity: actual_environment.finish_hash(),
            });
        }
        Ok(())
    }

    #[tracing::instrument(name = "pse.result.completion", skip_all, fields(pse.owner.stage = true))]
    pub(super) fn capture_completion(
        &self,
        attempt: Option<pse_model::generated::identities::AttemptId>,
    ) -> Result<Completion, WorkflowError> {
        let mut product = Completion {
            assessments: self.assessments.clone(),
            diagnostics: self.capture_diagnostics(),
            ..Default::default()
        };
        let facts = self.completed_facts();
        product.permission = Some(ScientificPermission::assess(
            None,
            match &self.outcome {
                super::RunOutcome::Initialization {
                    result: Ok(report), ..
                } => report.completed,
                _ => self.report().is_ok(),
            },
            facts.requested,
            facts.observed,
            &self.assessments,
        ));
        if matches!(self.original_request(), super::RunRequest::Modeling(_)) {
            product.step_permissions = self
                .assessments
                .iter()
                .map(|a| {
                    let mut permission = ScientificPermission::assess(
                        Some(a.step),
                        a.permits_result,
                        1,
                        1,
                        std::slice::from_ref(a),
                    );
                    permission.reference_point = match self.original_request() {
                        super::RunRequest::Modeling(requests) => (0..requests.len())
                            .find(|ordinal| self.modeling_step(*ordinal) as i64 == a.step)
                            .and_then(|ordinal| self.modeling_result(ordinal)),
                        _ => None,
                    }
                        .filter(|result| matches!(&result.outcome, Outcome::Native(native) if native.candidate.is_some()))
                        .map(|result| result.run_id.as_id());
                    permission
                })
                .collect();
        }
        if let super::RunOutcome::Initialization {
            result: Ok(report), ..
        } = &self.outcome
        {
            product.step_permissions = report.attempts.iter().enumerate().map(|(ordinal, attempt)| {
                let assessment = self.assessments.iter().filter(|fact| fact.step == ordinal as i64).cloned().collect::<Vec<_>>();
                let original = matches!(attempt.step, super::ModelingInitializationStep::Original);
                let mut permission = ScientificPermission::assess(
                    Some(ordinal as i64), report.completed && original,
                    1, usize::from(attempt.result.is_ok()), &assessment,
                );
                permission.reference_point = attempt.result.as_ref().ok()
                    .filter(|result| matches!(&result.outcome, Outcome::Native(native) if native.candidate.is_some()))
                    .map(|result| result.run_id.as_id());
                permission
            }).collect();
        }
        let environment = pse_math::context()
            .map_err(super::math)?
            .environment
            .identity();
        match &self.outcome {
            super::RunOutcome::Horizon {
                execution: super::HorizonRunOutcome::Compact(_),
                ..
            } => {
                return if self.original_completion_captured {
                    self.completion.clone().map_err(WorkflowError::Shared)
                } else {
                    Ok(product)
                };
            }
            super::RunOutcome::Modeling {
                request: requests, ..
            }
            | super::RunOutcome::Horizon {
                execution: super::HorizonRunOutcome::Full { requests, .. },
                ..
            } => self.capture_modeling_completion(&mut product, requests, environment)?,
            super::RunOutcome::Initialization { result, .. } => {
                if let Ok(report) = result {
                    let requests = report
                        .attempts
                        .iter()
                        .filter_map(|attempt| attempt.result.as_ref().ok())
                        .map(|result| result.prepared.clone())
                        .collect::<Vec<_>>();
                    self.capture_modeling_completion(&mut product, &requests, environment)?;
                }
            }
            super::RunOutcome::Simulation { request: p, result } => {
                let trajectory = result.as_ref().ok().map(Box::as_ref);
                if let Some(t) = trajectory {
                    product.accuracy_goals.extend(
                        t.completion()
                            .accuracy
                            .iter()
                            .map(|g| g.row(self.run_id, 0)),
                    );
                } else {
                    product
                        .accuracy_goals
                        .extend(p.numerics().policy.goals.iter().cloned().map(|goal| {
                            pse_math::engineering_accuracy::GoalResult::unavailable(
                                goal,
                                AccuracyUnavailableReason::MissingEvidence,
                            )
                            .row(self.run_id, 0)
                        }));
                }
                let profile_identity = match trajectory {
                    Some(t) => t.header().profile_identity,
                    None => super::dynamics::profile_identity(p.profile())
                        .map_err(crate::math::MathRuntimeError::from)?
                        .as_id(),
                };
                product.computation = Some(if let Some(trajectory) = trajectory {
                    trajectory.header().clone()
                } else {
                    let mut row = header(
                        self,
                        ComputationKind::Simulation,
                        p.source.revision.identity().as_id(),
                        profile_identity,
                    );
                    row.backend = p
                        .profile()
                        .resolved_method()
                        .ok()
                        .and_then(|method| pse_backend_native::dynamics::backend(method).ok());
                    row
                });
                let mut actual_environment =
                    FramedHasher::new(pse_ids::Frame::CompletedEnvironmentV1);
                actual_environment.hash(&environment).str(
                    &self.services.outer_attestation().map_or_else(
                        || "local-unattested".into(),
                        |attestation| attestation.build.to_hex(),
                    ),
                );
                #[cfg(feature = "solver-diffsol")]
                if trajectory.is_some() {
                    actual_environment.str(&pse_backend_native::dynamics::settings_identity(
                        p.profile(),
                    ));
                }
                let lineage = p.solved.lineage();
                product.lineage.push(run_lineage::Row {
                    run_id: self.run_id,
                    step: 0,
                    model_id: lineage.model_id,
                    revision: p.source.revision.identity().as_id(),
                    case_id: lineage.case_id,
                    instance_id: lineage.instance_id,
                    fit_id: lineage.fit_id,
                    request_identity: p.identity(),
                    preparation_identity: p.identity(),
                    profile_identity,
                    numerical_identity: p.numerics().key,
                    physical_identity: p.source.physical.key,
                    environment_identity: actual_environment.finish_hash(),
                });
            }
            #[cfg(feature = "solver-diffsol")]
            super::RunOutcome::Shooting {
                problem: p,
                initial,
                result,
            } => {
                let report = result.as_ref().ok().map(Box::as_ref);
                if let Some(r) = report {
                    product
                        .accuracy_goals
                        .extend(r.completion.accuracy.iter().map(|g| g.row(self.run_id, 0)));
                } else {
                    product
                        .accuracy_goals
                        .extend(p.numerics().policy.goals.iter().cloned().map(|goal| {
                            pse_math::engineering_accuracy::GoalResult::unavailable(
                                goal,
                                AccuracyUnavailableReason::MissingEvidence,
                            )
                            .row(self.run_id, 0)
                        }));
                }
                let native = report.and_then(|r| r.solve.as_ref());
                let trajectory = report.and_then(|r| r.trajectory.as_ref());
                let source = &p.simulation.source;
                let mut row = header(
                    self,
                    ComputationKind::Shooting,
                    source.revision.identity().as_id(),
                    p.profile_key.as_id(),
                );
                row.state = if native.is_some() {
                    NativeRunState::Native
                } else if report.is_some() {
                    NativeRunState::ConstantEvaluation
                } else {
                    NativeRunState::Rejected
                };
                row.termination = native.map(|r| r.termination.category);
                row.trajectory_termination = trajectory.map(|r| r.termination);
                row.backend = native.map(|r| r.backend);
                row.native_code = native.map(|r| r.termination.code);
                row.native_status = native.map(|r| r.termination.name.clone());
                row.qualification = report
                    .filter(|r| r.completion.permits_use())
                    .map_or(NativeQualification::Unqualified, |_| {
                        native.map_or(NativeQualification::Feasible, |r| r.qualification)
                    });
                row.candidate_kind = native.and_then(|r| r.candidate.as_ref().map(|c| c.kind));
                row.candidate_available = report.is_some_and(|r| r.candidate.is_some());
                if native.is_none() && row.candidate_available {
                    row.candidate_kind = Some(NativeCandidateKind::ConstantEvaluation);
                }
                row.feasible = report.and_then(|r| r.quality.as_ref().map(|q| q.feasible()));
                row.completed_time = trajectory
                    .map(|r| r.completed_time)
                    .filter(|t| t.is_finite());
                row.completed_samples = trajectory.map(|r| r.coverage.produced() as i64);
                row.validation_error = native
                    .and_then(|r| r.validation_failure().map(ToString::to_string))
                    .or_else(|| {
                        report.and_then(|r| r.validation_error.as_ref().map(ToString::to_string))
                    });
                row.error = row
                    .error
                    .or_else(|| trajectory.and_then(|r| r.error.as_ref().map(ToString::to_string)));
                product.computation = Some(row);
                let mut actual_environment =
                    FramedHasher::new(pse_ids::Frame::CompletedEnvironmentV1);
                actual_environment
                    .hash(&environment)
                    .str(&self.services.outer_attestation().map_or_else(
                        || "local-unattested".into(),
                        |attestation| attestation.build.to_hex(),
                    ))
                    .str(&pse_backend_native::dynamics::settings_identity(
                        p.simulation.profile(),
                    ));
                if let Some(native) = native {
                    actual_environment.str(native.backend.as_str());
                    for (name, value) in &native.provenance {
                        actual_environment.str(name).str(value);
                    }
                }
                let lineage = p.simulation.solved.lineage();
                product.lineage.push(run_lineage::Row {
                    run_id: self.run_id,
                    step: 0,
                    model_id: lineage.model_id,
                    revision: source.revision.identity().as_id(),
                    case_id: lineage.case_id,
                    instance_id: lineage.instance_id,
                    fit_id: lineage.fit_id,
                    request_identity: p.request_identity(initial.as_deref()).as_id(),
                    preparation_identity: p.request_identity(None).as_id(),
                    profile_identity: p.profile_key.as_id(),
                    numerical_identity: p.numerics().key,
                    physical_identity: source.physical.key,
                    environment_identity: actual_environment.finish_hash(),
                });
            }
            super::RunOutcome::Fit { request: p, result } => {
                let r = result.as_ref().ok().map(Box::as_ref);
                let retained = r.and_then(|r| r.completion.as_ref());
                if let Some(completion) = retained {
                    product
                        .accuracy_goals
                        .extend(completion.accuracy.iter().map(|g| g.row(self.run_id, 0)));
                } else {
                    product.accuracy_goals.extend(
                        p.problem.numerics.policy.goals.iter().cloned().map(|goal| {
                            pse_math::engineering_accuracy::GoalResult::unavailable(
                                goal,
                                AccuracyUnavailableReason::MissingEvidence,
                            )
                            .row(self.run_id, 0)
                        }),
                    );
                }
                let native = r.and_then(|r| r.solve.as_ref());
                let residual = r.and_then(|r| r.residual.as_ref());
                let physical = p.source.physical.key;
                let p = &p.problem;
                let mut row = header(
                    self,
                    ComputationKind::Fit,
                    p.source_identity,
                    p.profile_key.as_id(),
                );
                row.state = if native.is_some() || residual.is_some() {
                    NativeRunState::Native
                } else if r.is_some() {
                    NativeRunState::ConstantEvaluation
                } else {
                    NativeRunState::Rejected
                };
                row.termination = native.map(|s| s.termination.category).or_else(|| {
                    residual.map(super::fitting::FitResidualReport::termination_category)
                });
                row.backend = native.map(|s| s.backend);
                row.native_code = native.map(|s| s.termination.code);
                row.native_status = native
                    .map(|s| s.termination.name.clone())
                    .or_else(|| residual.map(|r| format!("residual_lm.{:?}", r.termination)));
                row.qualification = native
                    .map(|s| s.qualification)
                    .or_else(|| residual.map(|r| r.qualification))
                    .unwrap_or(NativeQualification::Unqualified);
                row.candidate_kind = native.and_then(|s| s.candidate.as_ref().map(|c| c.kind));
                row.candidate_available = r.is_some_and(|r| r.candidate.is_some());
                if residual.is_some() && row.candidate_available {
                    row.candidate_kind = Some(NativeCandidateKind::FinalIterate);
                }
                if native.is_none() && residual.is_none() && row.candidate_available {
                    row.candidate_kind = Some(NativeCandidateKind::ConstantEvaluation);
                }
                row.feasible = r.and_then(|r| r.quality.as_ref().map(|q| q.feasible()));
                row.estimate_qualified = r.map(super::FitReport::estimate_qualified);
                row.response_available = r.map(|r| r.responses.is_some());
                row.response_rank = r.and_then(|r| r.rank.map(|v| v as i64));
                row.response_condition = r.and_then(super::FitReport::response_condition);
                row.validation_error = native
                    .and_then(|s| s.validation_failure().map(ToString::to_string))
                    .or_else(|| r.and_then(|r| r.assessment.failure().map(|d| d.to_string())));
                row.error = row
                    .error
                    .or_else(|| {
                        residual.and_then(|r| {
                            r.failure
                                .as_ref()
                                .map(|failure| failure.cause().to_string())
                        })
                    })
                    .or_else(|| r.and_then(|r| r.diagnostic.as_ref().map(ToString::to_string)));
                product.computation = Some(row);
                let mut actual_environment =
                    FramedHasher::new(pse_ids::Frame::CompletedEnvironmentV1);
                actual_environment.hash(&environment).str(
                    &self.services.outer_attestation().map_or_else(
                        || "local-unattested".into(),
                        |attestation| attestation.build.to_hex(),
                    ),
                );
                if let Some(native) = native {
                    actual_environment.str(native.backend.as_str());
                    for (name, value) in &native.provenance {
                        actual_environment.str(name).str(value);
                    }
                }
                if residual.is_some() {
                    actual_environment
                        .str("residual_lm")
                        .str("e67066cf1351e91e054c30fcb2ff49d005db1e50");
                }
                let lineage = p.lineage.lineage();
                product.lineage.push(run_lineage::Row {
                    run_id: self.run_id,
                    step: 0,
                    model_id: lineage.model_id,
                    revision: p.source_identity,
                    case_id: lineage.case_id,
                    instance_id: lineage.instance_id,
                    fit_id: lineage.fit_id,
                    request_identity: p.key,
                    preparation_identity: p.key,
                    profile_identity: p.profile_key.as_id(),
                    numerical_identity: p.numerics.key,
                    physical_identity: physical,
                    environment_identity: actual_environment.finish_hash(),
                });
            }
        }
        self.capture_member_permissions(&mut product, attempt)?;
        self.capture_product_permissions(&mut product);
        Ok(product)
    }
}
/// Frame the start a native step actually used: its submitted seed by content (never by
/// the run that produced it), the predecessor step, a partial start, and whether the step
/// reused retained native state.
fn frame_start(h: &mut FramedHasher, native: Option<&pse_backend_native::solve::SolveReport>) {
    let Some(native) = native else {
        h.bool(false);
        return;
    };
    h.bool(true).bool(native.evidence.reused_native_state);
    let Some(receipt) = &native.start_receipt else {
        h.bool(false);
        return;
    };
    h.bool(true).bool(receipt.submitted);
    h.u64(receipt.previous_attempt.map_or(0, |a| a as u64 + 1));
    match receipt.seed.as_ref().filter(|_| receipt.submitted) {
        Some(seed) => {
            h.bool(true).hash(&seed.content_key());
        }
        None => {
            h.bool(false);
        }
    }
    let partial = receipt.sparse_seed.as_ref();
    h.u64(partial.map_or(0, |p| p.len() as u64));
    for (id, value) in partial.into_iter().flatten() {
        h.id(id).u64(value.to_bits());
    }
}
pub(super) fn simulation_header(
    run_id: pse_model::generated::identities::RunId,
    prepared: &super::ModelingSimulation,
    report: &pse_backend_native::dynamics::Report,
    checks: &super::modeling::dynamics::checks::SampleChecks,
    completion: &super::numerics::Completed,
) -> Result<computation_runs::Row, WorkflowError> {
    let profile = super::dynamics::profile_identity(prepared.profile())
        .map_err(crate::math::MathRuntimeError::from)?;
    let mut row = empty_header(
        run_id,
        ComputationKind::Simulation,
        prepared.source.revision.identity().as_id(),
        profile.as_id(),
    );
    row.state = NativeRunState::Native;
    row.trajectory_termination = Some(report.termination);
    row.backend = prepared
        .profile()
        .resolved_method()
        .ok()
        .and_then(|method| pse_backend_native::dynamics::backend(method).ok());
    row.candidate_available = report.endpoint.is_some();
    row.completed_time = report
        .completed_time
        .is_finite()
        .then_some(report.completed_time);
    row.completed_samples = Some(report.coverage.produced() as i64);
    row.feasible = checks
        .complete
        .then_some(checks.rows.iter().all(|c| c.satisfied));
    row.qualification = if completion.permits_use() {
        NativeQualification::Feasible
    } else {
        NativeQualification::Unqualified
    };
    row.validation_error = checks.error.as_ref().map(ToString::to_string);
    row.error = report.error.as_ref().map(ToString::to_string);
    row.response_available = Some(
        report
            .samples
            .iter()
            .any(|s| !s.output_sensitivities.is_empty()),
    );
    Ok(row)
}
fn header(
    result: &RunResult,
    kind: ComputationKind,
    source_identity: ContentHash,
    profile_identity: ContentHash,
) -> computation_runs::Row {
    let mut row = empty_header(result.run_id, kind, source_identity, profile_identity);
    row.error = result.error().map(ToString::to_string);
    row
}
fn empty_header(
    run_id: pse_model::generated::identities::RunId,
    kind: ComputationKind,
    source_identity: ContentHash,
    profile_identity: ContentHash,
) -> computation_runs::Row {
    computation_runs::Row {
        run_id,
        kind,
        source_identity,
        profile_identity,
        state: NativeRunState::Rejected,
        termination: None,
        trajectory_termination: None,
        backend: None,
        native_code: None,
        native_status: None,
        qualification: NativeQualification::Unqualified,
        candidate_kind: None,
        candidate_available: false,
        feasible: None,
        completed_time: None,
        completed_samples: None,
        estimate_qualified: None,
        response_available: None,
        response_rank: None,
        response_condition: None,
        validation_error: None,
        error: None,
    }
}

impl RunResult {
    /// Retained resumable native start of a selected step; omission selects the first step.
    pub fn available_start(
        &self,
        step: Option<usize>,
    ) -> Option<pse_backend_native::solve::WarmStart> {
        let step = step.unwrap_or_default();
        self.completed_facts()
            .native_reports
            .into_iter()
            .find_map(|(ordinal, report)| {
                (ordinal == step)
                    .then(|| report.warm_start.clone())
                    .flatten()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_backend_native::{
        OracleContract, ProblemError, quality,
        solve::{
            Assurance, Backend, Candidate, CandidateKind, Controls, Execution,
            NativeTermination as NativeStop, Qualification, SolveReport, Termination,
        },
    };
    use pse_ids::SemanticId;
    use pse_model::diagnostic::BoundaryClass;

    fn assessment(permitted: bool) -> pse_model::generated::runtime::candidate_assessments::Row {
        pse_model::generated::runtime::candidate_assessments::Row {
            run_id: pse_model::generated::identities::RunId::from_bytes([3; 16]),
            step: 0,
            native_termination: None,
            numerically_feasible: Some(true),
            closure: ClosureAssessment::Closed,
            policy: ClosurePolicy::RequireClosed,
            usability: if permitted {
                CandidateUse::Usable
            } else {
                CandidateUse::Unusable
            },
            reason: "independent permission fixture".into(),
            incumbent_policy: IncumbentPolicy::Refuse,
            candidate_kind: None,
            qualification: None,
            validated: None,
            bound_origin: None,
            bound: None,
            absolute_gap: None,
            relative_gap: None,
            qualifiers: vec![CandidateQualifier::ApplicabilityExtrapolationAllowed],
            refusals: vec![],
            permits_result: permitted,
            permits_seed: false,
        }
    }

    #[test]
    fn completion_permission_requires_original_report_coverage_and_every_assessment() {
        let accepted = assessment(true);
        let allowed =
            ScientificPermission::assess(None, true, 1, 1, std::slice::from_ref(&accepted));
        assert!(allowed.permits_result());
        assert_eq!(
            allowed.qualifiers,
            vec![CandidateQualifier::ApplicabilityExtrapolationAllowed]
        );
        for fact in [
            ScientificPermission::assess(None, false, 1, 1, std::slice::from_ref(&accepted)),
            ScientificPermission::assess(None, true, 2, 1, std::slice::from_ref(&accepted)),
            ScientificPermission::assess(None, true, 0, 0, &[]),
            ScientificPermission::assess(None, true, 1, 1, &[]),
            ScientificPermission::assess(None, true, 2, 2, &[accepted, assessment(false)]),
        ] {
            assert!(!fact.permits_result());
            assert!(!fact.refusals.is_empty());
        }
    }

    #[test]
    fn completion_and_product_permission_preserve_qualified_unclosed_scope() {
        let mut original = assessment(true);
        original.usability = CandidateUse::QualifiedUnclosed;
        original.closure = ClosureAssessment::Unclosed;
        original.qualifiers = vec![CandidateQualifier::ClosureAllowed];
        let fact = ScientificPermission::assess(None, true, 1, 1, &[original]);
        assert!(fact.permits_result());
        assert_eq!(fact.decision, CandidateUse::QualifiedUnclosed);
        let completion = Completion {
            permission: Some(fact),
            ..Default::default()
        };
        let product = completion.derive_permission(
            ProductScope::Task,
            DerivedQuantity::ParameterCovariance,
            super::super::local_analysis::record(Ok(&()), None, false),
        );
        assert!(product.permits_result());
        assert_eq!(product.decision, CandidateUse::QualifiedUnclosed);
    }

    #[cfg(feature = "engineering-service")]
    #[test]
    fn scientific_snapshots_preserve_governing_scope_and_independent_local_validity() {
        use buffa::Message as _;
        use buffa_descriptor::{DescriptorPool, DynamicMessage, Value};
        use pse_contract::boundary_projection::ConstructionRole;
        use std::sync::Arc;

        fn child(message: &DynamicMessage, tag: u32) -> &DynamicMessage {
            let Some(Value::Message(child)) = message.field_by_number(tag) else {
                panic!("expected message at independently specified tag {tag}");
            };
            child
        }
        fn inspect(name: &str, bytes: &[u8]) -> DynamicMessage {
            let pool = Arc::new(
                DescriptorPool::decode(crate::host_operations::engineering::generated::DESCRIPTORS)
                    .unwrap(),
            );
            let message = pool.message_index(name).unwrap();
            DynamicMessage::decode(pool, message, bytes).unwrap()
        }
        let graph = crate::host_operations::engineering_contract::projection().unwrap();
        for name in [
            "ScientificPermissionSnapshot",
            "ScientificProductPermissionSnapshot",
            "ScientificMemberPermissionSnapshot",
        ] {
            assert_eq!(
                graph.definitions()[name].construction,
                ConstructionRole::ObservationOnly
            );
        }
        let point = SemanticId::from_bytes([91; 16]);
        let mut original = assessment(true);
        original.usability = CandidateUse::QualifiedUnclosed;
        original.qualifiers = vec![CandidateQualifier::ClosureAllowed];
        let mut qualified = ScientificPermission::assess(Some(7), true, 1, 1, &[original]);
        qualified.reference_point = Some(point);
        let mut incomplete =
            ScientificPermission::assess(Some(7), false, 2, 1, &[assessment(true)]);
        incomplete.reference_point = Some(point);
        for (permission, expected_decision, permitted) in [
            (Some(qualified), "qualified_unclosed", true),
            (Some(incomplete), "unusable", false),
            (None, "unusable", false),
        ] {
            let completion = Completion {
                step_permissions: permission.into_iter().collect(),
                ..Default::default()
            };
            let mut validity = super::super::local_analysis::record(Ok(&()), None, false);
            validity.condition_1norm = Some(-0.0);
            validity.root_neighborhood = Some("original:neighborhood".into());
            let scope = ProductScope::Point {
                step: Some(7),
                member: None,
                point,
            };
            let product = completion.derive_permission(
                scope.clone(),
                DerivedQuantity::ParametricSensitivity,
                validity,
            );
            assert_eq!(product.permits_result(), permitted);
            assert!(
                product.validity.certified,
                "local success cannot replace original permission"
            );
            assert_eq!(
                serde_json::to_value(&scope).unwrap(),
                serde_json::json!({
                    "scope": "point", "step": 7, "member": null, "point": point,
                })
            );
            let native = serde_json::to_value(&product).unwrap();
            assert_eq!(native["decision"], expected_decision);
            assert_eq!(native["validity"]["certified"], true);
            let bytes =
                crate::engineering_projection::encode_scientific_product_permission_snapshot(
                    &product,
                )
                .unwrap()
                .encode_to_vec();
            let independent = inspect(
                "pse.engineering.v1.ScientificProductPermissionSnapshot",
                &bytes,
            );
            let point_scope = child(child(&independent, 1), 3);
            assert_eq!(point_scope.field_by_number(1), Some(&Value::I64(7)));
            assert_eq!(point_scope.field_by_number(2), None);
            assert_eq!(
                point_scope.field_by_number(3),
                Some(&Value::Bytes(vec![91; 16]))
            );
            assert_eq!(
                child(&independent, 2).field_by_number(1),
                Some(&Value::String("parametric_sensitivity".into()))
            );
            let local = child(&independent, 3);
            assert_eq!(local.field_by_number(1), Some(&Value::Bool(true)));
            assert_eq!(
                local.field_by_number(9),
                Some(&Value::U64((-0.0f64).to_bits()))
            );
            assert_eq!(
                local.field_by_number(16),
                Some(&Value::String("original:neighborhood".into()))
            );
            assert_eq!(
                child(&independent, 5).field_by_number(1),
                Some(&Value::String(expected_decision.into()))
            );
            if let Some(permission) = completion.step_permissions.first() {
                let bytes = crate::engineering_projection::encode_scientific_permission_snapshot(
                    permission,
                )
                .unwrap()
                .encode_to_vec();
                let independent =
                    inspect("pse.engineering.v1.ScientificPermissionSnapshot", &bytes);
                assert_eq!(
                    independent.field_by_number(1),
                    Some(&Value::Bytes(vec![91; 16]))
                );
                assert_eq!(independent.field_by_number(2), Some(&Value::I64(7)));
                assert_eq!(
                    independent.field_by_number(3),
                    Some(&Value::U64(if permitted { 1 } else { 2 }))
                );
                assert_eq!(independent.field_by_number(4), Some(&Value::U64(1)));
                assert_eq!(
                    child(&independent, 6).field_by_number(1),
                    Some(&Value::String(expected_decision.into()))
                );
            }
        }
    }

    #[test]
    fn product_permission_retains_original_and_accepted_pin_qualifiers() {
        let completion = Completion {
            permission: Some(ScientificPermission::assess(
                None,
                true,
                1,
                1,
                &[assessment(true)],
            )),
            ..Default::default()
        };
        let mut product = completion.derive_permission(
            ProductScope::Task,
            DerivedQuantity::ProfileInterval,
            super::super::local_analysis::record(Ok(&()), None, false),
        );
        product.retain_qualifiers([
            CandidateQualifier::ApplicabilityUnknownAllowed,
            CandidateQualifier::ClosureAllowed,
            CandidateQualifier::ApplicabilityUnknownAllowed,
        ]);
        assert!(product.permits_result());
        assert_eq!(product.decision, CandidateUse::Usable);
        assert_eq!(
            product.qualifiers,
            [
                CandidateQualifier::ApplicabilityExtrapolationAllowed,
                CandidateQualifier::ApplicabilityUnknownAllowed,
                CandidateQualifier::ClosureAllowed,
            ]
        );
        let mut encoded = serde_json::to_value(&product).unwrap();
        assert_eq!(
            serde_json::from_value::<ProductPermission>(encoded.clone())
                .unwrap()
                .qualifiers,
            product.qualifiers
        );
        encoded.as_object_mut().unwrap().remove("qualifiers");
        assert!(serde_json::from_value::<ProductPermission>(encoded).is_err());
    }

    #[test]
    fn local_validity_cannot_grant_permission_without_original_scope() {
        let validity = super::super::local_analysis::record(Ok(&()), None, false);
        let mut completion = Completion::default();
        assert!(
            !completion
                .derive_permission(
                    ProductScope::Task,
                    DerivedQuantity::ParameterCovariance,
                    validity.clone()
                )
                .permits_result()
        );
        completion.permission = Some(ScientificPermission::assess(
            None,
            true,
            1,
            1,
            &[assessment(true)],
        ));
        assert!(
            completion
                .derive_permission(
                    ProductScope::Task,
                    DerivedQuantity::ParameterCovariance,
                    validity
                )
                .permits_result()
        );
        let withheld = super::super::local_analysis::record::<()>(
            Err((
                WithheldReason::NoCandidate,
                "absent mathematical candidate".into(),
            )),
            None,
            false,
        );
        assert!(
            !completion
                .derive_permission(
                    ProductScope::Task,
                    DerivedQuantity::ParameterCovariance,
                    withheld
                )
                .permits_result()
        );
    }

    #[test]
    fn ambiguous_original_step_never_grants_product_permission() {
        let point = SemanticId::from_bytes([93; 16]);
        let mut accepted = ScientificPermission::assess(Some(0), true, 1, 1, &[assessment(true)]);
        accepted.reference_point = Some(point);
        let mut refused = ScientificPermission::assess(Some(0), false, 1, 0, &[]);
        refused.reference_point = Some(point);
        let validity = super::super::local_analysis::record(Ok(&()), None, false);
        for original in [
            vec![accepted.clone(), refused.clone()],
            vec![refused, accepted.clone()],
            vec![accepted.clone(), accepted.clone()],
        ] {
            let completion = Completion {
                permission: Some(ScientificPermission::assess(
                    None,
                    true,
                    1,
                    1,
                    &[assessment(true)],
                )),
                step_permissions: original,
                ..Default::default()
            };
            // The original observations remain readable, including local success. Neither their
            // order nor a successful task decision can resolve an ambiguous independent scope.
            assert_eq!(completion.step_permissions.len(), 2);
            assert!(completion.step_permission(0).is_none());
            assert!(completion.step_permission(1).is_none());
            let product = completion.derive_permission(
                ProductScope::Point {
                    step: Some(0),
                    member: None,
                    point,
                },
                DerivedQuantity::ParametricSensitivity,
                validity.clone(),
            );
            assert!(product.validity.certified);
            assert!(!product.permits_result());
            assert!(product.qualifiers.is_empty());
        }
        let completion = Completion {
            step_permissions: vec![accepted],
            ..Default::default()
        };
        assert!(completion.step_permission(0).unwrap().permits_result());
        assert!(
            completion
                .derive_permission(
                    ProductScope::Point {
                        step: Some(0),
                        member: None,
                        point,
                    },
                    DerivedQuantity::ParametricSensitivity,
                    validity,
                )
                .permits_result()
        );
    }

    #[test]
    fn point_products_use_exact_original_point_and_independent_step_permission() {
        let point = SemanticId::from_bytes([91; 16]);
        let other_point = SemanticId::from_bytes([92; 16]);
        let mut accepted = ScientificPermission::assess(Some(0), true, 1, 1, &[assessment(true)]);
        accepted.reference_point = Some(point);
        let failed = ScientificPermission::assess(Some(1), false, 1, 0, &[]);
        let mut completion = Completion {
            permission: Some(ScientificPermission::assess(None, false, 2, 1, &[])),
            step_permissions: vec![accepted, failed],
            ..Default::default()
        };
        let scope = ProductScope::Point {
            step: Some(0),
            member: None,
            point,
        };
        let validity = super::super::local_analysis::record(Ok(&()), None, false);
        let fact = completion.derive_permission(
            scope.clone(),
            DerivedQuantity::ParametricSensitivity,
            validity.clone(),
        );
        assert!(
            fact.permits_result(),
            "a failed sibling cannot erase the qualified original point"
        );
        completion.products.push(fact);
        let different = ProductScope::Point {
            step: Some(0),
            member: None,
            point: other_point,
        };
        assert!(
            !completion
                .derive_permission(
                    different.clone(),
                    DerivedQuantity::ParametricSensitivity,
                    validity.clone()
                )
                .permits_result()
        );
        assert!(
            completion
                .product_permission(&different, DerivedQuantity::ParametricSensitivity)
                .is_none()
        );
        let absent = ProductScope::Point {
            step: Some(1),
            member: None,
            point,
        };
        assert!(
            !completion
                .derive_permission(
                    absent,
                    DerivedQuantity::ParametricSensitivity,
                    validity.clone()
                )
                .permits_result()
        );
        assert!(
            !completion
                .derive_permission(
                    ProductScope::Step { step: 0 },
                    DerivedQuantity::ParametricSensitivity,
                    validity.clone()
                )
                .permits_result(),
            "step provenance alone cannot supply a point-derived product"
        );
        assert!(
            !completion
                .derive_permission(
                    ProductScope::Step { step: 99 },
                    DerivedQuantity::ParametricSensitivity,
                    validity
                )
                .permits_result()
        );
        let encoded = serde_json::to_value(&completion.products[0]).unwrap();
        assert_eq!(
            serde_json::from_value::<ProductPermission>(encoded)
                .unwrap()
                .scope,
            scope
        );
        completion.products.push(completion.products[0].clone());
        assert!(
            completion
                .product_permission(&scope, DerivedQuantity::ParametricSensitivity)
                .is_none(),
            "duplicate keys cannot select an arbitrary product"
        );
    }

    #[test]
    fn completion_current_version_requires_product_structure_and_never_defaults_permission_to_success()
     {
        let mut value = serde_json::to_value(Completion::default()).unwrap();
        assert!(serde_json::from_value::<Completion>(value.clone()).is_ok());
        value.as_object_mut().unwrap().remove("permission");
        // An optional field alone is not sufficient proof; absent original permission still refuses.
        let decoded = serde_json::from_value::<Completion>(value.clone()).unwrap();
        assert!(!decoded.permits_result());
        value.as_object_mut().unwrap().remove("products");
        assert!(serde_json::from_value::<Completion>(value).is_err());
    }

    #[tokio::test]
    async fn validation_failure_keeps_solve_projection_nullable_and_unqualified() {
        let runtime = super::super::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package p { def Root { param x:Scalar=2; eq square:x*x==4; } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, super::super::tests::physical())
            .await
            .unwrap();
        let prepared = package
            .prepare_solve(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                pse_kernels::DerivativeOrder::First,
                super::super::tests::compiler_profile(),
                super::super::tests::profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let run_id = pse_model::generated::identities::RunId::from_bytes([7; 16]);
        let owner = runtime
            .shared
            .math()
            .reserve("completion-unit", 4096)
            .unwrap();
        for (failure, class) in [
            (
                ProblemError::numerical("original observation"),
                BoundaryClass::Numerical,
            ),
            (
                ProblemError::memory("original observation allowance"),
                BoundaryClass::ResourceLimit,
            ),
        ] {
            // Boundary fixture: source preparation supplies real lineage, while this
            // report exercises projection only and is not scientific solve evidence.
            let contract = OracleContract {
                identity: ContentHash::from_bytes([1; 32]),
                variables: vec![],
                rows: vec![],
                derivatives: pse_kernels::DerivativeOrder::First,
                smoothness: pse_kernels::DerivativeOrder::First,
            };
            let mut report = SolveReport::new(
                Backend::Clarabel,
                &contract,
                NativeStop {
                    code: 1,
                    name: "Solved".into(),
                    message: None,
                    category: Termination::Success,
                    assurance: Assurance::NativeOptimal,
                },
                &Execution::new(
                    pse_diagnostics::Cancellation::default(),
                    &Controls::default(),
                ),
            );
            report.candidate = Some(Candidate {
                kind: CandidateKind::FinalIterate,
                primal: vec![],
                objective: None,
                row_dual: None,
                bound_dual: None,
                reduced_costs: None,
                slacks: None,
                commitment: None,
            });
            report.quality = Some(quality::Quality::new(vec![], vec![], vec![]).unwrap());
            report.qualification = Qualification::OptimalWithinTolerance;
            report.record_validation_failure(failure);
            let outcome = Outcome::Native(Box::new(report));
            let point = super::super::modeling::assessment::AssessedPoint {
                accuracy: Vec::new(),
                original: None,
                canonical_fallback: false,
                work: Vec::new(),
                values: prepared.model.values.clone(),
                checks: vec![],
                reports: vec![],
                provider_selections: None,
                error: None,
                unavailable_cause: None,
                complete: true,
                required_closure: 0,
                owner: owner.clone(),
            };
            let completion = point.completion(&outcome, &prepared.solve.numerics().policy);
            let step = super::super::ModelingResult::from_assessment(
                &runtime.preparation_services(),
                prepared.clone(),
                run_id,
                0,
                outcome,
                point,
                completion,
                owner.clone(),
                None,
            );
            let mut result = RunResult::joined(
                run_id,
                runtime.execution_services(),
                crate::workflow::RunOutcome::Modeling {
                    request: vec![prepared.clone()],
                    result: Ok(vec![step]),
                },
                None,
            );
            result.assessments = result.assess_candidates();
            let completion = result.capture_completion(None).unwrap();
            let [solve] = completion.solves.as_slice() else {
                panic!("one solve expected")
            };
            assert_eq!(solve.feasible, None);
            assert_eq!(solve.qualification, Qualification::Unqualified);
            assert_eq!(solve.assurance, Assurance::None);
            assert_eq!(solve.termination, Some(Termination::Success));
            assert_eq!(solve.native_code, Some(1));
            assert_eq!(solve.candidate_kind, Some(CandidateKind::FinalIterate));
            assert!(solve.validation_error.is_some());
            let [assessment] = completion.assessments.as_slice() else {
                panic!("one assessment expected")
            };
            assert_eq!(assessment.numerically_feasible, None);
            assert!(
                assessment
                    .refusals
                    .contains(&CandidateRefusal::FeasibilityUnavailable)
            );
            assert!(!assessment.refusals.contains(&CandidateRefusal::Infeasible));
            assert!(!assessment.permits_result);
            assert!(!assessment.permits_seed);
            assert!(completion.diagnostics.iter().any(|d| d.class == class));
        }
    }
}
