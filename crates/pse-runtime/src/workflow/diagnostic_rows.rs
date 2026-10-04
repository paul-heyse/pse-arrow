// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One semantic projection for every registry-owned diagnostic envelope.
use pse_diagnostics::DiagnosticObservationKind as K;
use pse_ids::SemanticId;
use pse_model::{
    diagnostic::{BoundaryDiagnostic, NonfiniteObservation as N, Observation},
    generated::enums::ModelingRealValueKind as R,
};

fn real_evidence(value: f64) -> (R, Option<f64>) {
    if value.is_finite() {
        (R::Finite, Some(value))
    } else if value == f64::INFINITY {
        (R::PositiveInfinity, None)
    } else if value == f64::NEG_INFINITY {
        (R::NegativeInfinity, None)
    } else {
        (R::Indeterminate, None)
    }
}

fn flatten<'a>(
    children: &'a [BoundaryDiagnostic],
    path: &mut Vec<i64>,
    result: &mut Vec<(Vec<i64>, &'a BoundaryDiagnostic)>,
) {
    for (index, child) in (0_i64..).zip(children) {
        path.push(index);
        result.push((path.clone(), child));
        flatten(&child.causes, path, result);
        path.pop();
    }
}

macro_rules! envelope {
    ($f:ident, $ctor:ident, $observation:ident, $contract:ident, $axis:ident, $location:ident, $validity:ident, $applicability:ident, $claim:ident, $input:ident, $permission:ident; $($extra:ident: $value:expr),* $(,)?) => {{
    $ctor { $($extra: $value,)*
code: $f.code, class: $f.class, severity: $f.severity, stage: $f.stage, rule: $f.rule, sources: $f.sources.clone(),
observations: $f.observations.iter().map(|(name,value)| {
    let mut row = $observation { name:name.clone(), kind:K::Missing, real:None, real_kind:None, integer:None, boolean:None, text:None, quantity:None, unit:None, context:None, contracts:Vec::new() };
    match value {
        Observation::Missing => {},
        Observation::Real(value) => {row.kind=K::Real;row.real=Some(*value);row.real_kind=Some(R::Finite);},
        Observation::Nonfinite(value) => {row.kind=K::Nonfinite;row.real_kind=Some(match value {N::Nan=>R::Indeterminate,N::PositiveInfinity=>R::PositiveInfinity,N::NegativeInfinity=>R::NegativeInfinity});},
        Observation::Physical(value)=>{row.kind=K::Physical;row.real=Some(value.magnitude);row.real_kind=Some(R::Finite);row.quantity=Some(value.quantity);row.unit=Some(value.unit);row.context=Some(value.context);},
        Observation::Integer(value)=>{row.kind=K::Integer;row.integer=Some(*value);},
        Observation::Boolean(value)=>{row.kind=K::Boolean;row.boolean=Some(*value);},
        Observation::Text(value)=>{row.kind=K::Text;row.text=Some(value.clone());},
        Observation::Contracts(contracts)=>{row.kind=K::Contracts;row.contracts=contracts.iter().map(|contract| $contract {quantity:SemanticId::from_bytes(contract.quantity),indices:contract.indices.iter().map(|axis|$axis{bound_index:SemanticId::from_bytes(axis[0]),domain:SemanticId::from_bytes(axis[1]),kind:SemanticId::from_bytes(axis[2])}).collect()}).collect();},
    }
    row
}).collect(),
locations:$f.locations.iter().map(|l|$location{source_id:l.source,revision:l.revision,path:l.path.clone(),name:l.name.clone(),start:l.start.map(i64::from),end:l.end.map(i64::from)}).collect(),
validity:$f.validity.as_ref().map(|v|$validity{layer:v.layer,source_id:v.source,form_id:v.form,set_ids:v.sets.clone(),variables:v.variables.iter().map(|v|i64::from(*v)).collect(),member_ids:v.members.clone()}),
applicability:$f.applicability.iter().map(|o|$applicability{
    claim:$claim{id:o.claim.id,coverage:o.claim.coverage,owner:o.claim.owner,owner_lineage:o.claim.owner_lineage.clone(),evidence:o.claim.evidence,form:o.claim.form,call:o.claim.call,records:o.claim.records.clone(),dependencies:o.claim.dependencies.clone(),layer:o.claim.layer,basis:o.claim.basis,reason:o.claim.reason.clone()},
    instance:o.instance,inputs:o.inputs.iter().map(|i|{let (real_kind,value)=real_evidence(i.value);$input{name:i.name.clone(),value,real_kind,quantity_type:i.quantity_type}}).collect(),
    outcome:o.outcome,required:o.required,permissions:o.permissions.iter().map(|v|$permission{id:v.id,scope:v.scope,target_kind:v.target_kind,targets:v.targets.clone(),allow_unknown:v.allow_unknown,allow_extrapolation:v.allow_extrapolation}).collect(),
    unknown_allowed:o.unknown_allowed,extrapolation_allowed:o.extrapolation_allowed,admitted:o.admitted,
}).collect(),
}
}};
}

mod finding {
    use super::*;
    use pse_model::generated::runtime::modeling_findings::{
        RuntimeModelingFindingsFieldApplicabilityItem as ApplicabilityRow,
        RuntimeModelingFindingsFieldApplicabilityItemClaim as ClaimRow,
        RuntimeModelingFindingsFieldApplicabilityItemInputsItem as InputRow,
        RuntimeModelingFindingsFieldApplicabilityItemPermissionsItem as PermissionRow,
        RuntimeModelingFindingsFieldCausesItem as Cause,
        RuntimeModelingFindingsFieldCausesItemApplicabilityItem as CauseApplicabilityRow,
        RuntimeModelingFindingsFieldCausesItemApplicabilityItemClaim as CauseClaimRow,
        RuntimeModelingFindingsFieldCausesItemApplicabilityItemInputsItem as CauseInputRow,
        RuntimeModelingFindingsFieldCausesItemApplicabilityItemPermissionsItem as CausePermissionRow,
        RuntimeModelingFindingsFieldCausesItemLocationsItem as CauseLocationRow,
        RuntimeModelingFindingsFieldCausesItemObservationsItem as CauseObservationRow,
        RuntimeModelingFindingsFieldCausesItemObservationsItemContractsItem as CauseContractRow,
        RuntimeModelingFindingsFieldCausesItemObservationsItemContractsItemIndicesItem as CauseAxisRow,
        RuntimeModelingFindingsFieldCausesItemValidity as CauseValidityRow,
        RuntimeModelingFindingsFieldLocationsItem as LocationRow,
        RuntimeModelingFindingsFieldObservationsItem as ObservationRow,
        RuntimeModelingFindingsFieldObservationsItemContractsItem as ContractRow,
        RuntimeModelingFindingsFieldObservationsItemContractsItemIndicesItem as AxisRow,
        RuntimeModelingFindingsFieldValidity as ValidityRow,
        RuntimeModelingFindingsRow as Envelope,
    };
    pub(super) fn project(
        run_id: pse_model::generated::identities::RunId,
        ordinal: i64,
        f: &BoundaryDiagnostic,
    ) -> Envelope {
        let mut flattened = Vec::new();
        flatten(&f.causes, &mut Vec::new(), &mut flattened);
        let causes = flattened.into_iter().map(|(tree_path, cause)| envelope!(cause, Cause, CauseObservationRow, CauseContractRow, CauseAxisRow, CauseLocationRow, CauseValidityRow, CauseApplicabilityRow, CauseClaimRow, CauseInputRow, CausePermissionRow; tree_path: tree_path)).collect();
        envelope!(f, Envelope, ObservationRow, ContractRow, AxisRow, LocationRow, ValidityRow, ApplicabilityRow, ClaimRow, InputRow, PermissionRow; run_id: run_id, ordinal: ordinal, causes: causes)
    }
}

mod study {
    use super::*;
    use pse_model::generated::runtime::study_outcomes::{
        RuntimeStudyOutcomesFieldDiagnostic as Envelope,
        RuntimeStudyOutcomesFieldDiagnosticApplicabilityItem as ApplicabilityRow,
        RuntimeStudyOutcomesFieldDiagnosticApplicabilityItemClaim as ClaimRow,
        RuntimeStudyOutcomesFieldDiagnosticApplicabilityItemInputsItem as InputRow,
        RuntimeStudyOutcomesFieldDiagnosticApplicabilityItemPermissionsItem as PermissionRow,
        RuntimeStudyOutcomesFieldDiagnosticCausesItem as Cause,
        RuntimeStudyOutcomesFieldDiagnosticCausesItemApplicabilityItem as CauseApplicabilityRow,
        RuntimeStudyOutcomesFieldDiagnosticCausesItemApplicabilityItemClaim as CauseClaimRow,
        RuntimeStudyOutcomesFieldDiagnosticCausesItemApplicabilityItemInputsItem as CauseInputRow,
        RuntimeStudyOutcomesFieldDiagnosticCausesItemApplicabilityItemPermissionsItem as CausePermissionRow,
        RuntimeStudyOutcomesFieldDiagnosticCausesItemLocationsItem as CauseLocationRow,
        RuntimeStudyOutcomesFieldDiagnosticCausesItemObservationsItem as CauseObservationRow,
        RuntimeStudyOutcomesFieldDiagnosticCausesItemObservationsItemContractsItem as CauseContractRow,
        RuntimeStudyOutcomesFieldDiagnosticCausesItemObservationsItemContractsItemIndicesItem as CauseAxisRow,
        RuntimeStudyOutcomesFieldDiagnosticCausesItemValidity as CauseValidityRow,
        RuntimeStudyOutcomesFieldDiagnosticLocationsItem as LocationRow,
        RuntimeStudyOutcomesFieldDiagnosticObservationsItem as ObservationRow,
        RuntimeStudyOutcomesFieldDiagnosticObservationsItemContractsItem as ContractRow,
        RuntimeStudyOutcomesFieldDiagnosticObservationsItemContractsItemIndicesItem as AxisRow,
        RuntimeStudyOutcomesFieldDiagnosticValidity as ValidityRow,
    };
    pub(super) fn project(f: &BoundaryDiagnostic) -> Envelope {
        let mut flattened = Vec::new();
        flatten(&f.causes, &mut Vec::new(), &mut flattened);
        let causes = flattened.into_iter().map(|(tree_path, cause)| envelope!(cause, Cause, CauseObservationRow, CauseContractRow, CauseAxisRow, CauseLocationRow, CauseValidityRow, CauseApplicabilityRow, CauseClaimRow, CauseInputRow, CausePermissionRow; tree_path: tree_path)).collect();
        envelope!(f, Envelope, ObservationRow, ContractRow, AxisRow, LocationRow, ValidityRow, ApplicabilityRow, ClaimRow, InputRow, PermissionRow;  causes: causes)
    }
}

mod attempt {
    use super::*;
    use pse_model::generated::runtime::study_outcomes::{
        RuntimeStudyOutcomesFieldAttemptsItemDiagnostic as Envelope,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticApplicabilityItem as ApplicabilityRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticApplicabilityItemClaim as ClaimRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticApplicabilityItemInputsItem as InputRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticApplicabilityItemPermissionsItem as PermissionRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItem as Cause,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItemApplicabilityItem as CauseApplicabilityRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItemApplicabilityItemClaim as CauseClaimRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItemApplicabilityItemInputsItem as CauseInputRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItemApplicabilityItemPermissionsItem as CausePermissionRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItemLocationsItem as CauseLocationRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItemObservationsItem as CauseObservationRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItemObservationsItemContractsItem as CauseContractRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItemObservationsItemContractsItemIndicesItem as CauseAxisRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticCausesItemValidity as CauseValidityRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticLocationsItem as LocationRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticObservationsItem as ObservationRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticObservationsItemContractsItem as ContractRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticObservationsItemContractsItemIndicesItem as AxisRow,
        RuntimeStudyOutcomesFieldAttemptsItemDiagnosticValidity as ValidityRow,
    };
    pub(super) fn project(f: &BoundaryDiagnostic) -> Envelope {
        let mut flattened = Vec::new();
        flatten(&f.causes, &mut Vec::new(), &mut flattened);
        let causes = flattened.into_iter().map(|(tree_path, cause)| envelope!(cause, Cause, CauseObservationRow, CauseContractRow, CauseAxisRow, CauseLocationRow, CauseValidityRow, CauseApplicabilityRow, CauseClaimRow, CauseInputRow, CausePermissionRow; tree_path: tree_path)).collect();
        envelope!(f, Envelope, ObservationRow, ContractRow, AxisRow, LocationRow, ValidityRow, ApplicabilityRow, ClaimRow, InputRow, PermissionRow;  causes: causes)
    }
}

mod profile {
    use super::*;
    use pse_model::generated::runtime::profile_points::{
        RuntimeProfilePointsFieldFailuresItem as Envelope,
        RuntimeProfilePointsFieldFailuresItemApplicabilityItem as ApplicabilityRow,
        RuntimeProfilePointsFieldFailuresItemApplicabilityItemClaim as ClaimRow,
        RuntimeProfilePointsFieldFailuresItemApplicabilityItemInputsItem as InputRow,
        RuntimeProfilePointsFieldFailuresItemApplicabilityItemPermissionsItem as PermissionRow,
        RuntimeProfilePointsFieldFailuresItemCausesItem as Cause,
        RuntimeProfilePointsFieldFailuresItemCausesItemApplicabilityItem as CauseApplicabilityRow,
        RuntimeProfilePointsFieldFailuresItemCausesItemApplicabilityItemClaim as CauseClaimRow,
        RuntimeProfilePointsFieldFailuresItemCausesItemApplicabilityItemInputsItem as CauseInputRow,
        RuntimeProfilePointsFieldFailuresItemCausesItemApplicabilityItemPermissionsItem as CausePermissionRow,
        RuntimeProfilePointsFieldFailuresItemCausesItemLocationsItem as CauseLocationRow,
        RuntimeProfilePointsFieldFailuresItemCausesItemObservationsItem as CauseObservationRow,
        RuntimeProfilePointsFieldFailuresItemCausesItemObservationsItemContractsItem as CauseContractRow,
        RuntimeProfilePointsFieldFailuresItemCausesItemObservationsItemContractsItemIndicesItem as CauseAxisRow,
        RuntimeProfilePointsFieldFailuresItemCausesItemValidity as CauseValidityRow,
        RuntimeProfilePointsFieldFailuresItemLocationsItem as LocationRow,
        RuntimeProfilePointsFieldFailuresItemObservationsItem as ObservationRow,
        RuntimeProfilePointsFieldFailuresItemObservationsItemContractsItem as ContractRow,
        RuntimeProfilePointsFieldFailuresItemObservationsItemContractsItemIndicesItem as AxisRow,
        RuntimeProfilePointsFieldFailuresItemValidity as ValidityRow,
    };
    pub(super) fn project(f: &BoundaryDiagnostic) -> Envelope {
        let mut flattened = Vec::new();
        flatten(&f.causes, &mut Vec::new(), &mut flattened);
        let causes = flattened.into_iter().map(|(tree_path, cause)| envelope!(cause, Cause, CauseObservationRow, CauseContractRow, CauseAxisRow, CauseLocationRow, CauseValidityRow, CauseApplicabilityRow, CauseClaimRow, CauseInputRow, CausePermissionRow; tree_path: tree_path)).collect();
        envelope!(f, Envelope, ObservationRow, ContractRow, AxisRow, LocationRow, ValidityRow, ApplicabilityRow, ClaimRow, InputRow, PermissionRow;  causes: causes)
    }
}

mod strategy {
    use super::*;
    use pse_model::generated::runtime::solve_strategy_events::{
        RuntimeSolveStrategyEventsFieldFailuresItem as Envelope,
        RuntimeSolveStrategyEventsFieldFailuresItemApplicabilityItem as ApplicabilityRow,
        RuntimeSolveStrategyEventsFieldFailuresItemApplicabilityItemClaim as ClaimRow,
        RuntimeSolveStrategyEventsFieldFailuresItemApplicabilityItemInputsItem as InputRow,
        RuntimeSolveStrategyEventsFieldFailuresItemApplicabilityItemPermissionsItem as PermissionRow,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItem as Cause,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItemApplicabilityItem as CauseApplicabilityRow,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItemApplicabilityItemClaim as CauseClaimRow,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItemApplicabilityItemInputsItem as CauseInputRow,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItemApplicabilityItemPermissionsItem as CausePermissionRow,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItemLocationsItem as CauseLocationRow,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItemObservationsItem as CauseObservationRow,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItemObservationsItemContractsItem as CauseContractRow,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItemObservationsItemContractsItemIndicesItem as CauseAxisRow,
        RuntimeSolveStrategyEventsFieldFailuresItemCausesItemValidity as CauseValidityRow,
        RuntimeSolveStrategyEventsFieldFailuresItemLocationsItem as LocationRow,
        RuntimeSolveStrategyEventsFieldFailuresItemObservationsItem as ObservationRow,
        RuntimeSolveStrategyEventsFieldFailuresItemObservationsItemContractsItem as ContractRow,
        RuntimeSolveStrategyEventsFieldFailuresItemObservationsItemContractsItemIndicesItem as AxisRow,
        RuntimeSolveStrategyEventsFieldFailuresItemValidity as ValidityRow,
    };
    pub(super) fn project(f: &BoundaryDiagnostic) -> Envelope {
        let mut flattened = Vec::new();
        flatten(&f.causes, &mut Vec::new(), &mut flattened);
        let causes = flattened.into_iter().map(|(tree_path, cause)| envelope!(cause, Cause, CauseObservationRow, CauseContractRow, CauseAxisRow, CauseLocationRow, CauseValidityRow, CauseApplicabilityRow, CauseClaimRow, CauseInputRow, CausePermissionRow; tree_path: tree_path)).collect();
        envelope!(f, Envelope, ObservationRow, ContractRow, AxisRow, LocationRow, ValidityRow, ApplicabilityRow, ClaimRow, InputRow, PermissionRow;  causes: causes)
    }
}

pub(super) fn project_finding(
    run_id: pse_model::generated::identities::RunId,
    ordinal: i64,
    diagnostic: &BoundaryDiagnostic,
) -> pse_model::generated::runtime::modeling_findings::Row {
    finding::project(run_id, ordinal, diagnostic)
}

pub(super) fn project_study_diagnostic(
    diagnostic: &BoundaryDiagnostic,
) -> pse_model::generated::runtime::study_outcomes::RuntimeStudyOutcomesFieldDiagnostic {
    study::project(diagnostic)
}

pub(super) fn project_study_attempt_diagnostic(
    diagnostic: &BoundaryDiagnostic,
) -> pse_model::generated::runtime::study_outcomes::RuntimeStudyOutcomesFieldAttemptsItemDiagnostic
{
    attempt::project(diagnostic)
}

/// Profile-chain typed failures projected through the registry-owned diagnostic envelope.
pub(crate) fn profile_failure(
    diagnostic: &BoundaryDiagnostic,
) -> pse_model::generated::runtime::profile_points::RuntimeProfilePointsFieldFailuresItem {
    profile::project(diagnostic)
}

/// Strategy failures use the same semantic envelope and ordered cause-tree projection.
pub(crate) fn strategy_failure(
    diagnostic: &BoundaryDiagnostic,
) -> pse_model::generated::runtime::solve_strategy_events::RuntimeSolveStrategyEventsFieldFailuresItem
{
    strategy::project(diagnostic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::ContentHash;
    use pse_model::diagnostic::{
        BoundaryClass, DiagnosticRule, DiagnosticStage, SourceLocation, ValidityLineage,
    };
    use pse_model::generated::{enums::ModelingValidityLayer, identities::RunId};

    #[test]
    fn every_relation_retains_physical_evidence_revision_and_ordered_causes() {
        let source = SemanticId::from_bytes([1; 16]);
        let revision = ContentHash::from_bytes([2; 32]);
        let mut diagnostic = BoundaryDiagnostic::new(
            BoundaryClass::InvalidModel,
            DiagnosticStage::Test,
            [source],
            DiagnosticRule::MathContract,
        );
        diagnostic.locations.push(SourceLocation {
            source,
            revision: Some(revision),
            path: "unit.pse".into(),
            name: Some("balance".into()),
            start: Some(5),
            end: Some(9),
        });
        diagnostic
            .observations
            .insert("missing".into(), Observation::Missing);
        diagnostic
            .observations
            .insert("nonfinite".into(), Observation::number(f64::NAN));
        diagnostic.observations.insert(
            "physical".into(),
            Observation::Physical(pse_model::diagnostic::PhysicalObservation {
                magnitude: 3.,
                quantity: source,
                unit: source,
                context: revision,
            }),
        );
        diagnostic.observations.insert(
            "operands".into(),
            Observation::Contracts(vec![pse_diagnostics::OperandContract {
                quantity: *source.as_bytes(),
                indices: vec![[*source.as_bytes(); 3]],
            }]),
        );
        diagnostic.validity = Some(ValidityLineage {
            layer: ModelingValidityLayer::Form,
            source,
            form: Some(source),
            sets: vec![source],
            variables: vec![2],
            members: vec![source],
        });
        let mut child = diagnostic.clone();
        child.causes = vec![diagnostic.clone()];
        diagnostic.causes = vec![child, diagnostic.clone()];
        let finding = project_finding(RunId::from(source), 0, &diagnostic);
        let study = project_study_diagnostic(&diagnostic);
        let attempt = project_study_attempt_diagnostic(&diagnostic);
        let profile = profile_failure(&diagnostic);
        let strategy = strategy_failure(&diagnostic);
        assert_eq!(finding.code, diagnostic.code);
        assert_eq!(study.code, finding.code);
        assert_eq!(attempt.code, finding.code);
        assert_eq!(profile.code, finding.code);
        assert_eq!(strategy.code, finding.code);
        assert_eq!(study.locations[0].revision, Some(revision));
        assert_eq!(attempt.validity.as_ref().unwrap().variables, vec![2]);
        for observation in &study.observations {
            match observation.kind {
                K::Physical => {
                    assert_eq!(observation.context, Some(revision));
                    assert_eq!(observation.quantity, Some(source));
                    assert_eq!(observation.real, Some(3.));
                }
                K::Contracts => assert_eq!(observation.contracts[0].indices[0].domain, source),
                K::Missing => assert!(observation.real.is_none()),
                K::Nonfinite => {
                    assert_eq!(observation.real_kind, Some(R::Indeterminate));
                    assert!(observation.real.is_none());
                }
                kind => panic!("unexpected observation {kind:?}"),
            }
        }
        let paths: Vec<_> = finding
            .causes
            .iter()
            .map(|cause| cause.tree_path.clone())
            .collect();
        assert_eq!(paths, vec![vec![0], vec![0, 0], vec![1]]);
        assert_eq!(
            study
                .causes
                .iter()
                .map(|cause| cause.tree_path.clone())
                .collect::<Vec<_>>(),
            paths
        );
        assert_eq!(
            attempt
                .causes
                .iter()
                .map(|cause| cause.tree_path.clone())
                .collect::<Vec<_>>(),
            paths
        );
        assert_eq!(attempt.causes[1].locations[0].revision, Some(revision));
        assert_eq!(profile.causes[1].locations[0].revision, Some(revision));
        assert_eq!(
            strategy.causes[1].validity.as_ref().unwrap().variables,
            vec![2]
        );
        for actual in [
            profile
                .causes
                .iter()
                .map(|cause| cause.tree_path.clone())
                .collect::<Vec<_>>(),
            strategy
                .causes
                .iter()
                .map(|cause| cause.tree_path.clone())
                .collect::<Vec<_>>(),
        ] {
            assert_eq!(actual, paths);
        }
    }

    #[test]
    fn profile_failure_roundtrip_preserves_two_level_causes_and_actual_terminal_flag() {
        use pse_model::generated::enums::{IntervalEnd, NativeTermination};
        use pse_relations::{columnar::RelationRow, generated::runtime::profile_points};
        let source = SemanticId::from_bytes([1; 16]);
        let mut diagnostic = BoundaryDiagnostic::new(
            BoundaryClass::InvalidModel,
            DiagnosticStage::Evaluation,
            [source],
            DiagnosticRule::MathContract,
        );
        let mut child = diagnostic.clone();
        child.sources = vec![SemanticId::from_bytes([2; 16])];
        let mut leaf = diagnostic.clone();
        leaf.sources = vec![SemanticId::from_bytes([3; 16])];
        leaf.observations
            .insert("attempts".into(), Observation::Integer(4));
        child.causes = vec![leaf];
        diagnostic.causes = vec![child];
        let row = profile_points::Row {
            run_id: RunId::from(source),
            parameter_id: source,
            end: IntervalEnd::Lower,
            point: 1,
            value: 2.0,
            seed: Some(0),
            qualification: None,
            termination: Some(NativeTermination::Evaluation),
            callback_terminal_failure: Some(true),
            failures: vec![profile_failure(&diagnostic)],
            objective: None,
            statistic: None,
            accepted: false,
            detail: Some("terminal callback".into()),
        };
        let registry = pse_schema::registry().unwrap();
        let validation = pse_relations::validate::ValidationContext::local(registry).unwrap();
        let mut builder = profile_points::Row::builder(registry, 1, &validation).unwrap();
        profile_points::Row::push(&mut builder, row.clone()).unwrap();
        let rows =
            profile_points::Row::rows(&profile_points::Row::finish(builder).unwrap()).unwrap();
        assert_eq!(rows, vec![row]);
        let actual = &rows[0].failures[0];
        assert_eq!(
            actual
                .causes
                .iter()
                .map(|cause| cause.tree_path.clone())
                .collect::<Vec<_>>(),
            vec![vec![0], vec![0, 0]]
        );
        assert_eq!(
            actual.causes[1].sources,
            vec![SemanticId::from_bytes([3; 16])]
        );
        assert_eq!(actual.causes[1].observations[0].integer, Some(4));
    }
}
