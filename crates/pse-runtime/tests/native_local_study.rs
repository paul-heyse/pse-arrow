// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Store-free local studies preserve scientific occurrence, source and detail ownership.
//! Requires solver-ipopt and force-validate; no canonical service or Python is used.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "public fixture construction and independent scientific assertions fail directly"
)]

#[path = "reference_support/mod.rs"]
mod support;

use futures_util::{FutureExt, future::BoxFuture};
use pse_backend_native::{presolve::PolicyKind, solve::{Backend, SolveIntent}};
use pse_diagnostics::{DiagnosticRule, DiagnosticStage};
use pse_ids::SemanticId;
use pse_model::{
    diagnostic::{BoundaryClass, Observation},
    generated::{enums::{ModelingAnalysisRoute, NumericalCoordinates, NumericalTarget, StudyPointState}, identities::DeclarationId},
    numerics::NumericalRequirement,
    scalars::FiniteBound,
    study::{Availability, Dependency, OccurrenceKey, StartPolicy, StudyLifecycle},
};
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::runtime::{solve_variables, study_outcomes},
};
use pse_runtime::{
    CancelSource,
    math::settings::SolveSettings,
    workflow::{
        BindingAssignment, BindingQuantity, BindingTarget, CaseOperation, LocalStudy,
        ModelingPackage, OperationRequest, OperationSource, PointOverlay, PreparationSettings,
        PreparedOperation, PreparedStudyOperation, RuntimeOperationRequest, SelectedStudyVariant,
        SourceChoice, SourceDelta, StudyDetailDemand, StudyInputs, StudyMemberDemand,
        StudyOccurrenceResult, StudyOutputDemand, StudyPoint, StudyPointPolicy, StudyReport,
        StudyVariants, WorkflowError,
        sampling::{DesignCoordinate, DesignKind, DesignRequest},
    },
};
use std::{panic::AssertUnwindSafe, sync::Arc};
use support::{NativeContext, Result};

// Same independent physical model as the retained Python local-study claims.
// The positive root is 2 at a=4, 3 at a=9; a=-1 has no real root.
const SOURCE: &str = r#"package algebraic { def Root {
  param a:Scalar=4; var x:Scalar; eq square:x*x==a;
  annotation start x(1); annotation bounds x(0,10);
  annotation report x("root"); annotation check x(x>1);
} }"#;

fn setup(context: &NativeContext) -> BoxFuture<'_, Result<(ModelingPackage, DeclarationId, SolveSettings)>> {
    async move {
        let package = context.package(SOURCE).await?;
        assert!(package.canonical_revision().is_none());
        assert!(context.runtime.canonical_store().is_err());
        let services = context.runtime.execution_services();
        let case = package.declarations(services.preparation()).await?.iter()
            .find(|row| row.name == "Root").unwrap().declaration_id;
        let mut settings = SolveSettings {
            intent: SolveIntent::Root, backend: Some(Backend::Ipopt),
            presolve: PolicyKind::Off, ..Default::default()
        };
        let mut preparation = PreparationSettings::default();
        preparation.compiler.evaluation.scratch_bytes = 1 << 20;
        let prepared = Box::pin(package.prepare_operation(
            services.preparation(), &OperationSource::of(&package), &preparation,
            RuntimeOperationRequest::DeclaredCase { case, settings: Box::new(settings.clone()) },
            None, &CancelSource::new(),
        )).await?;
        let PreparedOperation::Study(PreparedStudyOperation::DeclaredCase(prepared)) = prepared
            else { panic!("original declared square-root preparation required") };
        let solved = prepared.model.model.solved();
        // The physical-primitives fixture supplies no characteristic engineering
        // scale. Request explicit physical variable/equation budgets at the actual
        // admitted identities, rather than assuming a solver termination is accuracy.
        settings.numerics.requirements = prepared.solve.numerics().targets.iter()
            .filter(|target| matches!(target.kind, NumericalTarget::Variable | NumericalTarget::Row))
            .map(|target| NumericalRequirement {
                requirement_id: pse_ids::named_id(case.as_id(), &format!("local-study-accuracy:{:?}:{}",target.kind,target.id)),
                model_id: Some(solved.model()), case_id: solved.case(), instance_id: Some(solved.instance()),
                fit_id: None, target_id: target.id, target_kind: target.kind,
                nominal: None, scaling_factor: None, absolute_tolerance: Some(1e-8), relative_tolerance: Some(0.),
                shared_engineering_allowance: None, engineering_rule_id: None, unit_id: Some(target.unit),
                coordinates: NumericalCoordinates::Physical, priority: 0, required: true,
                provenance: "public local-study square root and original equation oracle".into(),
            }).collect();
        assert!(settings.numerics.requirements.iter().any(|target| target.target_kind == NumericalTarget::Variable));
        assert!(settings.numerics.requirements.iter().any(|target| target.target_kind == NumericalTarget::Row));
        drop(prepared);
        Ok((package, case, settings))
    }.boxed()
}

fn point(case: DeclarationId, settings: &SolveSettings, key: u32, overlay: PointOverlay) -> StudyPoint {
    let mut preparation = PreparationSettings::default();
    preparation.compiler.evaluation.scratch_bytes = 1 << 20;
    StudyPoint {
        operation: OperationRequest::DeclaredCase(CaseOperation {
            case, route: ModelingAnalysisRoute::Steady, settings: settings.clone(),
        }),
        preparation, overlay,
        policy: StudyPointPolicy { key: OccurrenceKey(key), dependencies: vec![], start: StartPolicy::Fresh, attempt_limit: 1 },
    }
}

fn assignment(package: &ModelingPackage, target: BindingTarget, value: f64) -> Result<PointOverlay> {
    let physical = package.physical_context();
    Ok(PointOverlay { assignments: vec![BindingAssignment {
        target,
        value: BindingQuantity {
            magnitude: FiniteBound::try_new(value)?,
            quantity: physical.resolve_quantity("Scalar")?, unit: physical.resolve_unit("dimensionless")?,
        },
    }] })
}

fn admit<'a>(context: &'a NativeContext, package: &'a ModelingPackage, points: Vec<StudyPoint>, output: StudyOutputDemand) -> BoxFuture<'a, Result<LocalStudy>> {
    async move { Ok(Box::pin(package.admit_study(
        &context.runtime.execution_services(), &StudyInputs::Explicit(points), &output, &CancelSource::new(),
    )).await?) }.boxed()
}

fn full_result(report: &StudyReport, index: usize) -> &Arc<pse_runtime::workflow::RunResult> {
    let StudyOccurrenceResult::Ephemeral(result) = report.results[index].as_ref().unwrap()
        else { panic!("store-free full occurrence must retain the joined native result") };
    assert!(report.results[index].as_ref().unwrap().stored_keys().is_none());
    result
}

fn assert_root(table: &FieldCheckedBatch, expected: f64, parameter: f64) -> solve_variables::Row {
    let rows = solve_variables::Row::rows(table).unwrap();
    let variables = rows.iter().filter(|row| !row.parameter).collect::<Vec<_>>();
    assert_eq!(variables.len(), 1);
    let variable = variables[0];
    let value = variable.value.unwrap();
    let tolerance = variable.tolerance.unwrap();
    assert!(value.is_finite());
    assert!(tolerance > 0. && tolerance <= 1e-8);
    assert!((value-expected).abs() <= tolerance, "{value} versus {expected}; tolerance={tolerance}");
    assert!(variable.quantity_id.is_some_and(|id| id != SemanticId::NIL));
    assert!(variable.unit_id.is_some_and(|id| id != SemanticId::NIL));
    let parameters = rows.iter().filter(|row| row.parameter).collect::<Vec<_>>();
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters[0].value.unwrap().to_bits(), parameter.to_bits());
    (*variable).clone()
}

async fn settle(context: &NativeContext, body: std::thread::Result<Result<()>>) -> Result<()> {
    let cleanup = context.close().await;
    match body {
        Ok(result) => { result?; cleanup }
        Err(panic) => { let _ = cleanup; std::panic::resume_unwind(panic) }
    }
}

#[tokio::test]
async fn generated_equal_local_occurrences_keep_distinct_keys_results_and_retained_source() -> Result<()> {
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let (package, case, settings) = setup(&context).await?;
        let services = context.runtime.execution_services();
        let physical = package.physical_context();
        let design = Box::pin(package.generate_design(services.preparation(), &DesignRequest {
            root: case, bindings: Default::default(),
            coordinates: vec![DesignCoordinate {
                target: BindingTarget::Path("a".into()), quantity: physical.resolve_quantity("Scalar")?,
                unit: physical.resolve_unit("dimensionless")?,
                range: Some([FiniteBound::try_new(4.)?,FiniteBound::try_new(4.)?]), levels: vec![],
            }],
            // The finite work account includes 128 units per coordinate plus
            // 16 per emitted cell: this one-coordinate, two-point grid needs 160.
            kind: DesignKind::Grid, count: 2, seed: 0, grid_levels: vec![2], maximum_work: 256,
        }, Default::default(), &CancelSource::new())).await?;
        assert_eq!(design.points().len(), 2);
        assert_eq!(design.points()[0], design.points()[1]);
        assert_eq!(design.attribution().count, 2);
        let points = design.points().iter().cloned().zip([7,11])
            .map(|(overlay,key)| point(case,&settings,key,overlay)).collect();
        let local = admit(&context,&package,points,StudyOutputDemand::Full {}).await?;
        assert_eq!(local.points[0].binding_hash,local.points[1].binding_hash);
        assert_eq!(local.points.iter().map(|point| point.policy.key.0).collect::<Vec<_>>(),[7,11]);
        assert!(matches!(Box::pin(context.runtime.record_study(&local,&CancelSource::new())).await, Err(WorkflowError::RecordingUnavailable)));
        drop((design,package));
        let error = Box::pin(local.execute(&services,1,&CancelSource::new())).await.unwrap_err();
        assert!(matches!(&error,WorkflowError::Input(_)),"{error}");
        let report = Box::pin(local.execute(&services,2,&CancelSource::new())).await?;
        drop(local);
        assert_eq!(report.decision.conclusion.availability,Availability::Complete);
        assert_eq!(report.decision.conclusion.lifecycle,StudyLifecycle::Terminal);
        assert!(report.diagnostic_selection().await?.is_none());
        assert_eq!(report.outcomes.len(),2);
        for (outcome,key) in report.outcomes.iter().zip([7,11]) {
            assert_eq!(outcome.key,OccurrenceKey(key));
            assert_eq!(outcome.lifecycle,StudyPointState::Completed);
            assert!(outcome.scientific.usable);
            assert!(outcome.diagnostic.is_none());
            assert_eq!(outcome.attempts.len(),1);
            assert!(outcome.attempts[0].attempt_id.is_none(),"local tries have no fabricated durable attempt");
        }
        let first = full_result(&report,0);
        let second = full_result(&report,1);
        assert!(first.usable() && second.usable());
        assert_ne!(first.run_id,second.run_id);
        let values = first.table("runtime.solve_variables")?;
        assert_root(&second.table("runtime.solve_variables")?,2.,4.);
        let history = report.table()?;
        drop(report);
        context.close().await?;
        let rows = study_outcomes::Row::rows(&history)?;
        assert_eq!(rows.iter().map(|row| row.point_index).collect::<Vec<_>>(),[7,11]);
        assert!(rows.iter().all(|row| row.usable && row.attempts.len()==1));
        assert_root(&values,2.,4.);
        Ok(())
    }).catch_unwind().await;
    settle(&context,body).await
}

#[tokio::test]
async fn local_binding_identity_and_physical_refusal_keep_original_member_lineage() -> Result<()> {
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let (package,case,settings) = setup(&context).await?;
        let by_path = admit(&context,&package,vec![point(case,&settings,2,
            assignment(&package,BindingTarget::Path("a".into()),9.)?)],StudyOutputDemand::Full {}).await?;
        let member = *by_path.points[0].binding.entries.keys().next().unwrap();
        let by_member = admit(&context,&package,vec![point(case,&settings,13,
            assignment(&package,BindingTarget::Member(member),9.)?)],StudyOutputDemand::Full {}).await?;
        assert_eq!(by_path.points[0].binding_hash,by_member.points[0].binding_hash);
        let a = &by_path.points[0].binding.entries[&member];
        let b = &by_member.points[0].binding.entries[&member];
        assert_eq!(a.canonical.into_inner(),9.);
        assert_eq!(b.canonical.into_inner(),9.);
        assert_ne!(a.supplied,b.supplied);
        let physical = package.physical_context();
        let length = physical.resolve_quantity("Length")?;
        let wrong = PointOverlay { assignments: vec![BindingAssignment {
            target: BindingTarget::Path("a".into()),
            value: BindingQuantity { magnitude: FiniteBound::try_new(1.)?, quantity: length, unit: physical.resolve_unit("m")? },
        }] };
        let error = admit(&context,&package,vec![point(case,&settings,17,wrong)],StudyOutputDemand::Full {}).await.unwrap_err();
        let error = error.downcast_ref::<WorkflowError>().unwrap();
        let diagnostic = error.boundary_diagnostic();
        assert_eq!(diagnostic.rule,DiagnosticRule::StudyBindingPhysical);
        assert_eq!(diagnostic.stage,DiagnosticStage::StudyBinding);
        assert_eq!(diagnostic.class,BoundaryClass::InvalidModel);
        let location = &diagnostic.locations[0];
        assert_eq!(location.path,"a");
        assert_eq!(location.source,member);
        assert!(diagnostic.sources.contains(&location.source));
        assert_eq!(location.revision,Some(OperationSource::of(&package).revision.as_id()));
        let Observation::Contracts(operands) = &diagnostic.observations["operands"]
            else { panic!("original expected and supplied contracts required") };
        assert_eq!(operands.len(),2);
        assert_eq!(operands[0].quantity,*physical.resolve_quantity("Scalar")?.as_id().as_bytes());
        assert_eq!(operands[1].quantity,*length.as_id().as_bytes());
        assert!(operands.iter().all(|operand| operand.indices.is_empty()));
        drop(package);
        let report = Box::pin(by_member.execute(&context.runtime.execution_services(),1,&CancelSource::new())).await?;
        assert!(report.outcomes[0].scientific.usable);
        assert_root(&full_result(&report,0).table("runtime.solve_variables")?,3.,9.);
        Ok(())
    }).catch_unwind().await;
    settle(&context,body).await
}

#[tokio::test]
async fn local_unusable_predecessor_refuses_without_dispatch_and_keeps_independent_result() -> Result<()> {
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let (package,case,settings) = setup(&context).await?;
        let mut failed_settings = settings.clone();
        failed_settings.controls.iterations = 50;
        let failed = point(case,&failed_settings,3,assignment(&package,BindingTarget::Path("a".into()),-1.)?);
        let mut dependent = point(case,&settings,9,PointOverlay::default());
        dependent.policy.dependencies = vec![Dependency::UsableResult(OccurrenceKey(3))];
        let independent = point(case,&settings,17,PointOverlay::default());
        let local = admit(&context,&package,vec![failed,dependent,independent],StudyOutputDemand::Full {}).await?;
        drop(package);
        let report = Box::pin(local.execute(&context.runtime.execution_services(),3,&CancelSource::new())).await?;
        assert_eq!(report.outcomes.len(),3);
        assert!(!report.outcomes[0].scientific.usable);
        assert!(!full_result(&report,0).usable(),"x*x=-1 cannot produce a usable real root");
        let refused = &report.outcomes[1];
        assert_eq!(refused.key,OccurrenceKey(9));
        assert_eq!(refused.lifecycle,StudyPointState::Failed);
        assert!(!refused.scientific.usable);
        assert!(refused.attempts.is_empty() && refused.start.is_none());
        assert_eq!(refused.diagnostic.as_ref().unwrap().rule,DiagnosticRule::StudyDependencyUnusable);
        assert!(report.results[1].is_none());
        assert!(report.outcomes[2].scientific.usable);
        assert_root(&full_result(&report,2).table("runtime.solve_variables")?,2.,4.);
        assert_eq!(report.decision.conclusion.availability,Availability::Partial);
        assert_eq!(report.decision.conclusion.lifecycle,StudyLifecycle::Terminal);
        let rows = study_outcomes::Row::rows(&report.table()?)?;
        assert!(rows[1].result_id.is_none() && rows[1].attempts.is_empty());
        assert_eq!(rows[1].diagnostic.as_ref().unwrap().rule,DiagnosticRule::StudyDependencyUnusable);
        Ok(())
    }).catch_unwind().await;
    settle(&context,body).await
}

#[tokio::test]
async fn local_lazy_selection_retains_source_and_compact_output_keeps_exact_or_empty_member() -> Result<()> {
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let (package,case,settings) = setup(&context).await?;
        let services = context.runtime.execution_services();
        let source = OperationSource::of(&package);
        let parent = package.revision_id().to_owned();
        let variants = StudyVariants {
            version: Default::default(), source: source.clone(),
            choices: vec![SourceChoice { alternatives: vec![
                SourceDelta { parent: parent.clone(), ..Default::default() },
                SourceDelta { parent: parent.clone(), removals: vec![DeclarationId::from_id(SemanticId::NIL)], ..Default::default() },
            ] }],
            selected: vec![SelectedStudyVariant { ordinal: 0, point: point(case,&settings,19,PointOverlay::default()) }],
        };
        let local = Box::pin(package.admit_study(&services,&StudyInputs::Lazy(Box::new(variants)),
            &StudyOutputDemand::Full {},&CancelSource::new())).await?;
        assert!(local.points.is_empty());
        assert_eq!(local.point_count(),1);
        let lazy = local.variants.as_ref().unwrap();
        assert_eq!(lazy.variants.source,source);
        assert_eq!(lazy.variants.selected[0].point.policy.key,OccurrenceKey(19));
        assert_eq!(lazy.variants.choices[0].alternatives[1].parent,parent);
        // Selecting the invalid alternative is retained as lazy intent too; it
        // must fail when executed, while the unselected edit cannot break ordinal0.
        let mut invalid = lazy.variants.clone();
        invalid.selected[0].ordinal = 1;
        let invalid = Box::pin(package.admit_study(&services,&StudyInputs::Lazy(Box::new(invalid)),
            &StudyOutputDemand::Full {},&CancelSource::new())).await?;
        let bad = Box::pin(invalid.execute(&services,1,&CancelSource::new())).await?;
        assert!(!bad.outcomes[0].scientific.usable);
        assert!(bad.outcomes[0].diagnostic.is_some());
        let baseline = Box::pin(local.execute(&services,1,&CancelSource::new())).await?;
        assert_eq!(baseline.outcomes[0].key,OccurrenceKey(19));
        assert!(baseline.outcomes[0].scientific.usable);
        let original_table = full_result(&baseline,0).table("runtime.solve_variables")?;
        let original = assert_root(&original_table,2.,4.);
        let mut compact = Vec::new();
        for (key,select_member) in [(23,true),(29,false)] {
            let output = StudyOutputDemand::Compact {
                detail: StudyDetailDemand {
                    relations: vec![], members: vec![StudyMemberDemand::Variable {
                        step: original.step + i64::from(!select_member), symbol_id: original.symbol_id,
                    }], maximum_rows: 1, maximum_bytes: 1 << 20,
                }, failed_detail: Default::default(),
            };
            compact.push((select_member,admit(&context,&package,vec![point(case,&settings,key,PointOverlay::default())],output).await?));
        }
        drop((baseline,invalid,package));
        let retained_source = Box::pin(local.execute(&services,1,&CancelSource::new())).await?;
        drop(local);
        assert_eq!(retained_source.outcomes[0].key,OccurrenceKey(19));
        assert!(retained_source.outcomes[0].scientific.usable);
        assert_root(&full_result(&retained_source,0).table("runtime.solve_variables")?,2.,4.);
        drop(retained_source);
        for (select_member,local) in compact {
            let mut report = Box::pin(local.execute(&services,1,&CancelSource::new())).await?;
            assert!(report.outcomes[0].scientific.usable);
            assert_eq!(report.decision.conclusion.availability,Availability::Complete);
            let StudyOccurrenceResult::Compact(result) = report.results[0].take().unwrap()
                else { panic!("explicit compact output required") };
            let completion = result.completion().unwrap();
            assert!(!completion.solves.is_empty() && !completion.assessments.is_empty());
            assert!(completion.permits_result());
            let mut cursor = result.into_table_cursor("runtime.solve_variables")?;
            assert_eq!(cursor.schema(),original_table.batch().schema());
            drop((report,local));
            let mut selected = Vec::new();
            while let Some(batch) = cursor.next_batch().await? { selected.extend(solve_variables::Row::rows(&batch)?); }
            assert_eq!(selected.len(),usize::from(select_member));
            if select_member {
                let row = &selected[0];
                assert_eq!(row.step,original.step);
                assert_eq!(row.symbol_id,original.symbol_id);
                assert!(!row.parameter);
                assert_eq!(row.quantity_id,original.quantity_id);
                assert_eq!(row.unit_id,original.unit_id);
                assert_eq!(row.value.unwrap().to_bits(),original.value.unwrap().to_bits());
                assert_eq!(row.tolerance,original.tolerance);
                assert!((row.value.unwrap()-2.).abs() <= row.tolerance.unwrap());
            }
        }
        Ok(())
    }).catch_unwind().await;
    settle(&context,body).await
}
