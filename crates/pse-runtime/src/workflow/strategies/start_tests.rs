// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::tests::profile;
use super::*;
use crate::math::solves::Outcome;
use crate::workflow::{RunReport,tests::{compiler_profile,id,physical,runtime}};
use std::collections::BTreeMap;

#[tokio::test]
async fn authored_sequence_separates_seed_policy_reuse_and_original_acceptance() {
    let runtime=runtime();
    let physical=physical();
    let neutral=physical.quantities().neutral_dimensionless().unwrap();
    let rows=pse_authoring::language::parse("package p {def Root {param threshold:Scalar=-3; var x:Scalar; eq square:x*x==4; annotation start x(1); annotation check x(x>threshold);}}",id(90),pse_authoring::language::IdentityPolicy::Named,Default::default()).unwrap();
    let root=rows.iter().find(|r|r.name=="Root").unwrap().declaration_id;
    let package=runtime.modeling_package(rows,physical,BTreeMap::from([("Scalar".into(),neutral)])).unwrap();
    let cancel=crate::CancelSource::new();
    let mut analysis=package.declared_analysis(root,pse_model::generated::enums::ModelingAnalysisRoute::Steady,compiler_profile(),profile(SolveIntent::Root),Default::default(),Default::default(),&cancel).await.unwrap();
    let positive=package.prepare_analysis(&analysis,&cancel).await.unwrap();
    analysis.case.values.insert("x".into(),-1.0);
    let negative=package.prepare_analysis(&analysis,&cancel).await.unwrap();
    for reuse in [ReusePolicy::Fresh,ReusePolicy::AllowRebuild,ReusePolicy::RequireReuse] {
        for policy in [StartPolicy::NoPriorStart,StartPolicy::PreviousAccepted] {
            analysis.solver.controls.start=policy;
            analysis.solver.controls.reuse=reuse;
            let next=package.prepare_analysis(&analysis,&cancel).await.unwrap();
            let result=runtime.start_modeling(vec![positive.clone(),next],false,&cancel).await.unwrap().wait().await.unwrap();
            let RunReport::Modeling(report)=result.report().unwrap() else {panic!()};
            assert_eq!(report.len(),2);
            assert!(report.iter().all(|r|r.accepted),"{report:?}");
            let Outcome::Native(second)=&report[1].outcome else {panic!()};
            let receipt=second.start_receipt.as_ref().unwrap();
            let expected=if policy==StartPolicy::PreviousAccepted {2.0} else {-2.0};
            assert!((second.candidate.as_ref().unwrap().primal[0]-expected).abs()<1e-6);
            assert_eq!(receipt.submitted,policy==StartPolicy::PreviousAccepted);
            assert_eq!(receipt.previous_attempt,(policy==StartPolicy::PreviousAccepted).then_some(0));
            let output=second.warm_start.as_ref().unwrap().origin.as_ref().unwrap();
            assert_eq!(output.run,Some(result.run_id));
            assert_eq!(output.attempt,1);
            let table=result.table("runtime.modeling_checks").unwrap();
            use pse_relations::columnar::RelationRow;
            let checks=pse_relations::generated::runtime::modeling_checks::Row::rows(&table).unwrap();
            assert!(checks.iter().any(|r|r.step==0));
            assert!(checks.iter().any(|r|r.step==1));
            assert!(checks.iter().all(|r|r.sample_index==0));
        }
    }
    analysis.solver.controls.start=StartPolicy::NoPriorStart;
    analysis.solver.controls.reuse=ReusePolicy::AllowRebuild;
    analysis.case.values.insert("threshold".into(),0.0);
    analysis.solver.numerics.closure=pse_model::generated::enums::ClosurePolicy::AllowUnclosed;
    let rejected=package.prepare_analysis(&analysis,&cancel).await.unwrap();
    for independent in [false,true] {
        analysis.case.values.insert("x".into(),1.0);
        analysis.solver.controls.start=StartPolicy::PreviousAccepted;
        let next=package.prepare_analysis(&analysis,&cancel).await.unwrap();
        let result=runtime.start_modeling(vec![rejected.clone(),next],independent,&cancel).await.unwrap().wait().await.unwrap();
        let RunReport::Modeling(report)=result.report().unwrap() else {panic!()};
        assert_eq!(report.len(),if independent {2} else {1});
        assert!(!report[0].accepted);
        assert!(matches!(&report[0].outcome,Outcome::Native(r) if r.quality.as_ref().unwrap().feasible()));
        assert_eq!(result.assessments().len(),2);
        assert!(!result.usable());
        if independent {
            assert!(report[1].accepted);
            let Outcome::Native(second)=&report[1].outcome else {panic!()};
            assert!(!second.start_receipt.as_ref().unwrap().submitted);
            assert!((second.candidate.as_ref().unwrap().primal[0]-2.0).abs()<1e-6);
        }
        assert_eq!(result.table("runtime.solve_runs").unwrap().batch().num_rows(),2);
    }
    let x=negative.model.case.compiled().plan.columns()[0];
    let result=negative.with_primal_start(BTreeMap::from([(x,1.0)])).unwrap().start().unwrap().wait().await.unwrap();
    let RunReport::Modeling(report)=result.report().unwrap() else {panic!()};
    assert!(matches!(&report[0].outcome,Outcome::Native(r) if (r.candidate.as_ref().unwrap().primal[0]-2.0).abs()<1e-6));
}

/// One authored root `x*x == 4` with `annotation start x(1)`.
async fn root_package() -> (
    Runtime,
    crate::workflow::ModelingPackage,
    crate::workflow::ModelingAnalysis,
    crate::CancelSource,
) {
    let runtime=runtime();
    let physical=physical();
    let neutral=physical.quantities().neutral_dimensionless().unwrap();
    let rows=pse_authoring::language::parse("package p {def Root {var x:Scalar; eq square:x*x==4; annotation start x(1);}}",id(91),pse_authoring::language::IdentityPolicy::Named,Default::default()).unwrap();
    let root=rows.iter().find(|r|r.name=="Root").unwrap().declaration_id;
    let package=runtime.modeling_package(rows,physical,BTreeMap::from([("Scalar".into(),neutral)])).unwrap();
    let cancel=crate::CancelSource::new();
    let analysis=package.declared_analysis(root,pse_model::generated::enums::ModelingAnalysisRoute::Steady,compiler_profile(),profile(SolveIntent::Root),Default::default(),Default::default(),&cancel).await.unwrap();
    (runtime,package,analysis,cancel)
}

#[tokio::test]
async fn native_option_change_keeps_seed_compatible() {
    use pse_backend_native::{execution::BackendSettings,kinsol,solve::SeedTransformation};
    let (runtime,package,mut analysis,cancel)=root_package().await;
    let positive=package.prepare_analysis(&analysis,&cancel).await.unwrap();
    // The next step changes a native setting and starts where the authored start leads
    // to the other root; only the predecessor's seed can take it to +2.
    analysis.solver.controls.start=StartPolicy::PreviousAccepted;
    analysis.solver.controls.reuse=ReusePolicy::AllowRebuild;
    analysis.solver.backend=BackendSettings::Kinsol(kinsol::Method{setup_interval:1,..Default::default()});
    analysis.case.values.insert("x".into(),-1.0);
    let next=package.prepare_analysis(&analysis,&cancel).await.unwrap();
    let (a,b)=(positive.solve.compatibility().unwrap(),next.solve.compatibility().unwrap());
    assert_eq!(a.layout,b.layout);
    assert_ne!(a.profile,b.profile);
    assert!(!a.same_session(b));
    let result=runtime.start_modeling(vec![positive,next],false,&cancel).await.unwrap().wait().await.unwrap();
    let RunReport::Modeling(report)=result.report().unwrap() else {panic!()};
    assert!(report.iter().all(|r|r.accepted),"{report:?}");
    let Outcome::Native(second)=&report[1].outcome else {panic!("{:?}",report[1].outcome)};
    let receipt=second.start_receipt.as_ref().unwrap();
    assert!(receipt.submitted);
    assert_eq!(receipt.previous_attempt,Some(0));
    assert!(matches!(receipt.transformations.as_slice(),[SeedTransformation::Normalization(_)]),"{:?}",receipt.transformations);
    assert!((second.candidate.as_ref().unwrap().primal[0]-2.0).abs()<1e-6);
    // The changed profile rebuilt the native session; only the seed carried over.
    assert!(!second.evidence.reused_native_state);
}

#[tokio::test]
async fn lineage_identity_changes_with_seed() {
    let (runtime,package,mut analysis,cancel)=root_package().await;
    let positive=package.prepare_analysis(&analysis,&cancel).await.unwrap();
    analysis.case.values.insert("x".into(),-1.0);
    let negative=package.prepare_analysis(&analysis,&cancel).await.unwrap();
    analysis.case.values.remove("x");
    analysis.solver.controls.start=StartPolicy::PreviousAccepted;
    let next=package.prepare_analysis(&analysis,&cancel).await.unwrap();
    let lineage=|steps:Vec<crate::workflow::ModelingSolvePreparation>|{
        let runtime=runtime.clone();
        let cancel=&cancel;
        async move {
            let result=runtime.start_modeling(steps,false,cancel).await.unwrap().wait().await.unwrap();
            let RunReport::Modeling(report)=result.report().unwrap() else {panic!()};
            assert!(report.iter().all(|r|r.accepted),"{report:?}");
            result.completion().unwrap().lineage.clone()
        }
    };
    let seeded_up=lineage(vec![positive.clone(),next.clone()]).await;
    let seeded_down=lineage(vec![negative,next.clone()]).await;
    let again=lineage(vec![positive,next]).await;
    // The same request, seeded differently, is a different lineage.
    assert_eq!(seeded_up[1].preparation_identity,seeded_down[1].preparation_identity);
    assert_eq!(seeded_up[1].profile_identity,seeded_down[1].profile_identity);
    assert_ne!(seeded_up[1].request_identity,seeded_down[1].request_identity);
    // The seed enters by content, never by the run that produced it.
    assert_ne!(seeded_up[1].run_id,again[1].run_id);
    assert_eq!(seeded_up[1].request_identity,again[1].request_identity);
}
