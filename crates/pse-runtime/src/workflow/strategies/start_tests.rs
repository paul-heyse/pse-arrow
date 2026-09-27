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
