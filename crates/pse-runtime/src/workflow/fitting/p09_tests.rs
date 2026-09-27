// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-clock, schedule and event behavior through authored fit experiments.
use super::*;
use crate::workflow::tests::{compiler_profile,id};
use native::{solve::{Backend,Execution},NlpOracle};
use std::sync::atomic::AtomicBool;
fn source(mixed: bool, expected: f64) -> (crate::workflow::ModelingPackage,FitProfile) {
    let mut physical=crate::workflow::tests::physical();
    physical.preconditions=Arc::new(pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap());
    physical.key=pse_compiler::workspace::physical_identity(&physical.quantities,&physical.preconditions);
    let scalar=physical.quantities.neutral_dimensionless().unwrap();
    let time=pse_quantity::QuantityTypeId::from_id(SemanticId::parse_hex("e2ccf6d0a394403db967f4f35b83cb7c").unwrap());
    let rows=pse_authoring::language::parse("package p { test Dynamic source \"analytic time integral\" revision \"1\" fixture {dof 0; run integrated; integrate samples(160{s},161{s}) relative(1e-8) normalized_absolute(1e-8) step(1e-5{s});} { domain t: Time from 160{s} to 161{s}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 2; var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == p; eq initial: x[160{s}] == 10{s}; let y[i in t]: Time = x[i]+i-100{s}; let hit[i in t]: Time = x[i]-11{s}; let jump[i in t]: Time = 20{s}; annotation report y(\"measurement\"); annotation check x(x[i] >= 10{s}); } def Steady { param p: Scalar = 2; let y: Scalar = p; annotation check p(p > 0); } }",id(20),pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
    let root=|name|rows.iter().find(|r|r.name==name).unwrap().declaration_id;
    let mut data=FitData::default();
    data.datasets.push(serde_json::from_value(serde_json::json!({"dataset_id":id(70),"name":"mixed","source":"analytic","content_hash":ContentHash::from_bytes([1;32])})).unwrap());
    for (obs,value,quantity) in [(71,expected,time),(72,1.,scalar)] {
        data.observations.push(serde_json::from_value(serde_json::json!({"observation_id":id(obs),"dataset_id":id(70),"target":"response","value":value,"unit_id":physical.quantities.quantity_type(quantity).unwrap().canonical_unit.as_id(),"std_dev":1.,"timestamp":null,"tag":null,"source_span":{"document_id":id(70),"start":0,"end":0}})).unwrap());
    }
    let mut fit:FitDeclaration=serde_json::from_value(serde_json::json!({"fit_id":id(73),"parameters":[{"symbol_id":id(3),"fixed":false,"value":2.,"lower":0.1,"upper":10.,"scale":1.}],"experiments":[{"experiment_id":id(74),"case_id":root("Dynamic"),"route":"integrated","bindings":[{"parameter_id":id(3),"path":"p"}]}],"observations":[{"observation_id":id(71),"experiment_id":id(74),"output_path":"y[160{s}]","time":161.,"time_basis":"model_clock","included":true,"importance":1.}]})).unwrap();
    if mixed {
        fit.experiments.push(serde_json::from_value(serde_json::json!({"experiment_id":id(75),"case_id":root("Steady"),"route":"steady","bindings":[{"parameter_id":id(3),"path":"p"}]})).unwrap());
        fit.observations.push(serde_json::from_value(serde_json::json!({"observation_id":id(72),"experiment_id":id(75),"output_path":"y","time":null,"included":true,"importance":1.})).unwrap());
    }
    data.fits.push(fit);
    let profile=FitProfile {
        solver:SolverProfile {intent:SolveIntent::Optimize,selection:native::solve::SolverSelection::Explicit(Backend::Ipopt),controls:native::solve::Controls{hessian:HessianMode::LimitedMemory,..Default::default()},presolve:native::presolve::Policy::Off,numerics:Default::default(),convexity:Default::default(),backend:native::execution::BackendSettings::Default},
        simulations:BTreeMap::from([(id(74),native::dynamics::Profile{start:160.,end:161.,samples:vec![160.,161.],parameter_scales:vec![1.],..Default::default()})]),
        modes:BTreeMap::new(),rank_tolerance:1e-8,max_cells:100000,
    };
    let runtime=crate::workflow::tests::runtime_with_workspace(32<<20);
    (runtime.modeling_package(rows,physical,BTreeMap::from([("Scalar".into(),scalar),("Time".into(),time)])).unwrap().with_fit_data(data).unwrap(),profile)
}
#[tokio::test]
async fn mixed_shared_parameter_gradient_uses_inline_forward_sensitivities() {
    let (package,profile)=source(true,73.);
    let cancel=crate::CancelSource::new();
    let mut exact=profile.clone(); exact.solver.controls.hessian=HessianMode::Exact;
    assert!(package.prepare_fit_problem(id(73),exact,compiler_profile(),Default::default(),&cancel).await.is_err());
    let (problem,_)=package.prepare_fit_problem(id(73),profile,compiler_profile(),Default::default(),&cancel).await.unwrap();
    assert_eq!(problem.contract.variables.len(),1);
    let execution=Execution::new(Arc::new(AtomicBool::new(false)),&problem.profile.solver.controls);
    let mut oracle=FitOracle::new(problem,execution).unwrap();
    assert!((oracle.objective(&[2.]).unwrap()-0.5).abs()<1e-6);
    let mut gradient=[0.]; oracle.gradient(&[2.],&mut gradient).unwrap();
    assert!((gradient[0]-1.).abs()<1e-6);
    let step=1e-4;
    let finite=(oracle.objective(&[2.+step]).unwrap()-oracle.objective(&[2.-step]).unwrap())/(2.*step);
    assert!((finite-gradient[0]).abs()<1e-5);
    let RankDiagnostic{responses,rank,..}=oracle.response_rank(&[2.]).unwrap();
    assert_eq!(rank,1); assert!((responses[(0,0)]-1.).abs()<1e-6); assert!((responses[(1,0)]-1.).abs()<1e-6);
}
#[cfg(feature="solver-ipopt")]
#[tokio::test]
async fn nonzero_clock_smooth_scheduled_and_state_reset_fits_share_response_contract() {
    for mode in 0..3 {
        let expected=match mode {0=>74.,1=>73.,_=>83.};
        let (package,mut profile)=source(false,expected);
        if mode==1 {profile.simulations.get_mut(&id(74)).unwrap().changes.push(native::dynamics::InputChange{time:160.5,parameters:vec![1.]});}
        if mode==2 {profile.modes.insert(id(74),vec![
            crate::workflow::ModelingDynamicMode{name:"before".into(),facts:BTreeMap::new(),events:vec![crate::workflow::ModelingDynamicEvent{guard:"hit[160{s}]".into(),reset:BTreeMap::from([("x[160{s}]".into(),"jump[160{s}]".into())]),terminal:false,next_mode:Some("after".into()),tolerance:1e-8}]},
            crate::workflow::ModelingDynamicMode{name:"after".into(),facts:BTreeMap::new(),events:vec![]},
        ]);}
        let cancel=crate::CancelSource::new();
        let (problem,_)=package.prepare_fit_problem(id(73),profile.clone(),compiler_profile(),Default::default(),&cancel).await.unwrap();
        assert_eq!(problem.measurements[0].time,Some(161.)); assert_eq!(problem.measurements[0].sample_index,Some(1));
        let execution=Execution::new(Arc::new(AtomicBool::new(false)),&problem.profile.solver.controls);
        let mut oracle=FitOracle::new(problem,execution).unwrap();
        let RankDiagnostic{responses,rank,..}=oracle.response_rank(&[2.]).unwrap();
        assert_eq!(rank,1); assert!((responses[(0,0)]-if mode==1{0.5}else{1.}).abs()<1e-5,"mode {mode}"); drop(oracle);
        let prepared=package.prepare_fit(id(73),profile,compiler_profile(),Default::default(),&cancel).await.unwrap();
        let result=prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report)=result.report().unwrap() else{panic!("missing fit")};
        assert!((report.candidate.as_ref().unwrap()[0]-3.).abs()<1e-4,"mode {mode}: {report:?}");
        let trajectory=&report.trajectories[&id(74)]; assert_eq!(trajectory.completed_time,161.);
        assert!((report.predictions[0].unwrap()-expected).abs()<1e-4);
        if mode==2 {assert!(!trajectory.events.is_empty());}
        assert!(report.checks_complete && report.checks.iter().all(|c|c.satisfied),"{report:?}");
        assert!(report.estimate_qualified(),"mode {mode}: {report:?}");
        assert!(result.usable());
        assert!(result.table("runtime.modeling_checks").unwrap().batch().num_rows()>=2);
    }
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn authored_integration_controls_bind_to_the_experiment_instance() {
    let (package, mut profile) = source(false, 74.);
    profile.simulations.clear();
    let prepared = package
        .prepare_fit(
            id(73),
            profile,
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing fit")
    };
    assert!(
        (report.candidate.as_ref().unwrap()[0] - 3.).abs() < 1e-4,
        "{report:?}"
    );
    assert!(result.usable(), "{report:?}");
}

#[tokio::test]
async fn transient_fit_deadline_is_time_limit() {
    let (package, mut profile) = source(false, 74.);
    profile.simulations.get_mut(&id(74)).unwrap().time_limit = std::time::Duration::from_nanos(1);
    let cancel = crate::CancelSource::new();
    let (problem, _) = package
        .prepare_fit_problem(
            id(73),
            profile,
            compiler_profile(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let problem = Arc::new(problem);
    let execution = Execution::new(
        Arc::new(AtomicBool::new(false)),
        &problem.profile.solver.controls,
    );
    let mut oracle = FitOracle::new(problem.clone(), execution.clone()).unwrap();
    // The Diffsol deadline stays a typed time limit, never an evaluation failure.
    let error = oracle.objective(&[2.]).unwrap_err();
    assert!(
        matches!(
            error,
            ProblemError::Limit {
                kind: native::LimitKind::Time,
                ..
            }
        ),
        "{error:?}"
    );
    assert_eq!(
        native::callback::classify(&error),
        native::callback::Failure::Stopped(native::solve::Termination::TimeLimit)
    );
    let mut state = native::callback::CallbackState::new(execution);
    assert!(
        state
            .evaluate("fit.objective", || oracle.objective(&[2.]))
            .is_none()
    );
    assert_eq!(
        state.terminal.as_ref().map(|t| t.0),
        Some(native::solve::Termination::TimeLimit)
    );
    assert_eq!(
        crate::workflow::diagnostics::observed(&error, "fit").class,
        pse_model::diagnostic::BoundaryClass::ResourceLimit
    );
    // Cancellation of the attempt is a cancellation.
    let cancelled = Execution::new(
        Arc::new(AtomicBool::new(true)),
        &problem.profile.solver.controls,
    );
    let mut oracle = FitOracle::new(problem, cancelled).unwrap();
    let error = oracle.objective(&[2.]).unwrap_err();
    assert!(matches!(error, ProblemError::Cancelled), "{error:?}");
    assert_eq!(
        native::callback::classify(&error),
        native::callback::Failure::Stopped(native::solve::Termination::Cancelled)
    );
}
