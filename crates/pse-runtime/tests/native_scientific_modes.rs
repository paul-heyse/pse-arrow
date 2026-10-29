// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public local scientific callers with analytic expectations and honest partial results.
//! Requires solver-ipopt, solver-highs and explicit correctness force-validation.
//! Independent external-reference exchanges remain separate integration targets.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "public scientific fixtures and independent expectations fail directly"
)]

#[path = "reference_support/mod.rs"]
mod reference_support;
#[allow(dead_code, unused_imports, reason = "reuse native candidate construction without the isolated reference exchange records")]
#[path = "reference_covariance_support/mod.rs"]
mod covariance;
#[allow(dead_code, unused_imports, reason = "reuse native candidate construction without the isolated reference exchange records")]
#[path = "reference_sensitivity_support/mod.rs"]
mod sensitivity;
#[allow(dead_code, unused_imports, reason = "reuse native candidate construction without the isolated reference exchange records")]
#[path = "reference_degeneracy_support/mod.rs"]
mod degeneracy;

use futures_util::FutureExt;
use pse_backend_native::solve::{Backend, Qualification, SolveIntent};
use pse_ids::SemanticId;
use pse_relations::{columnar::RelationRow, generated::{authored::fit_cases,
    runtime::{fit_observations, fit_parameters, modeling_checks, solve_runs, solve_strategy_events, solve_variables}}};
use pse_runtime::{
    CancelSource,
    math::settings::{SensitivityRequest, SolveSettings},
    workflow::{DiagnosticOperation, FitDeclarations, FitExecutionDocument, FitOptimizer,
        FitPreparationDocument, FitProfile, FitProfileDocument, FitRoute, JacobianDiagnosticControls,
        ModelingAnalysis, ModelingPackage, PreparedFit, RunReport, RunRequest},
};
use reference_support::{NativeContext, Result};
use std::{collections::{BTreeMap, BTreeSet}, panic::AssertUnwindSafe, time::Duration};

fn close_to(actual: f64, expected: f64, allowance: f64) {
    assert!(allowance.is_finite() && allowance > 0.);
    assert!(actual.is_finite() && (actual - expected).abs() <= allowance,
        "actual={actual}, analytic={expected}, allowance={allowance}");
}

fn finish<T>(
    outcome: std::thread::Result<Result<T>>,
    closed: Result<()>,
) -> Result<T> {
    match outcome {
        Ok(result) => { closed?; result }
        Err(panic) => { let _ = closed; std::panic::resume_unwind(panic) }
    }
}

fn measurement_fit<'a>(context: &'a NativeContext, fixed: bool)
    -> futures_util::future::BoxFuture<'a, Result<ModelingPackage>>
{
    Box::pin(async move {
        let package = context.package(
            "package fitting {def Measurement {param length:Length=2{m};
             annotation check length(length>0{m});}
             entity kind origin provenance {attribute title:Text;}
             entity origin experiment {title=\"known length\"}
             enum role {measured facets(measured)}
             entity kind reading {attribute value:Length?;attribute sigma:Length?;}
             @id(\"9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d\")
             entity reading measured provenance(experiment,role.measured)
             {value=2{m},sigma=0.1{m}}}"
        ).await?;
        let case = package.declarations(&context.runtime.preparation_services()).await?
            .iter().find(|row|row.name=="Measurement").unwrap().declaration_id;
        let mut declarations = FitDeclarations::default();
        declarations.fits.push(serde_json::from_value(serde_json::json!({
            "fit_id":SemanticId::from_bytes([158;16]),
            "parameters":[{"symbol_id":SemanticId::from_bytes([153;16]),"fixed":fixed,
                "value":if fixed {2.} else {1.},"lower":if fixed {Some(0.)} else {None},
                "upper":if fixed {Some(10.)} else {None},"scale":1.}],
            "experiments":[{"experiment_id":SemanticId::from_bytes([159;16]),"case_id":case,
                "route":"steady","bindings":[{"parameter_id":SemanticId::from_bytes([153;16]),"path":"length"}]}],
            "observations":[{"observation_id":SemanticId::from_bytes([157;16]),
                "value_attribute":"value","standard_deviation_attribute":"sigma",
                "experiment_id":SemanticId::from_bytes([159;16]),"output_path":"length",
                "time":null,"time_basis":null,"time_unit_id":null,"included":true,"importance":1.}]
        }))?);
        Ok(Box::pin(package.with_fit_declarations(&context.runtime.source_services(),declarations)).await?)
    })
}

fn measurement_profile(optimizer: FitOptimizer, backend: Option<Backend>) -> Result<FitProfile> {
    let document: FitPreparationDocument = serde_json::from_value(serde_json::json!({
        "optimizer":optimizer,"solver":SolveSettings {
            intent:SolveIntent::Optimize,backend,..Default::default()
        }
    }))?;
    // Exercise the actual public preparation document, without adding a parallel codec.
    let bytes = serde_json::to_vec(&document)?;
    let decoded: FitPreparationDocument = serde_json::from_slice(&bytes)?;
    assert_eq!(serde_json::to_value(&decoded)?,serde_json::to_value(&document)?);
    Ok(decoded.profile()?)
}

fn prepare_measurement<'a>(context: &'a NativeContext, package: &'a ModelingPackage,
    profile: FitProfile, cancel: &'a CancelSource)
    -> futures_util::future::BoxFuture<'a, Result<PreparedFit>>
{
    Box::pin(async move {
        Ok(Box::pin(package.prepare_fit(&context.runtime.preparation_services(),
            SemanticId::from_bytes([158;16]).into(),profile,Default::default(),Default::default(),cancel)).await?)
    })
}

#[tokio::test]
async fn native_residual_lm_reports_actual_stopping_and_original_fit_qualification() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = measurement_fit(&context,false).await?;
        let cancel = CancelSource::new();
        let prepared = prepare_measurement(&context,&package,
            measurement_profile(FitOptimizer::ResidualLm,None)?,&cancel).await?;
        assert_eq!(prepared.route(),FitRoute::ResidualLm);
        let original_identity = prepared.identity();
        let profile_identity = prepared.strategy_profile();
        let strategy = prepared.numerical_strategy();
        assert_eq!(strategy.mechanisms.len(),1);
        assert_eq!(strategy.mechanisms[0].profile.unwrap().method,
            pse_model::strategy::NumericalMethod::ResidualLm);
        assert!(!strategy.mechanisms[0].support.is_empty());
        let handle = prepared.start(&context.runtime.execution_services())?;
        let result = handle.wait().await?;
        assert_eq!(handle.wait().await?.run_id,result.run_id);
        let RunRequest::Fit(request) = result.request() else {panic!("original fit request retained")};
        assert_eq!(request.identity(),original_identity);
        assert_eq!(request.strategy_profile(),profile_identity);
        assert_eq!(request.route(),FitRoute::ResidualLm);
        let RunReport::Fit(report) = result.report().unwrap() else {panic!("actual residual fit report")};
        assert!(result.usable(),"{report:?}");
        assert!(report.solve.is_none(),"LM must not fabricate an NLP backend report");
        let FitExecutionDocument::ResidualLm {report:residual} = report.execution_document()
            else {panic!("actual residual-LM execution descriptor")};
        assert!(residual.failure.is_none());
        assert_eq!(residual.qualification,Qualification::Stationary);
        assert!(residual.stationarity.is_some_and(f64::is_finite));
        let stopping = residual.stopping.unwrap();
        assert_eq!(stopping.criteria.version(),1);
        for threshold in [stopping.criteria.relative_cost_change(),stopping.criteria.relative_step(),
            stopping.criteria.column_residual_cosine()] {
            assert_eq!(threshold,30.*f64::EPSILON);
        }
        assert!(residual.residual_evaluations>0 && residual.residual_evaluations<=stopping.residual_evaluation_limit);
        assert!(residual.parameter_updates<=stopping.parameter_update_limit);
        assert!(residual.jacobian_evaluations>0);
        assert!(report.checks_complete && report.checks.iter().all(|check|check.satisfied));
        assert!(report.estimate_qualified());
        assert!(result.completion().unwrap().diagnostics.is_empty());
        assert_eq!(serde_json::to_value(report.execution_document())?["method"],"residual_lm");
        let observations = result.table("runtime.fit_observations")?;
        let parameters = result.table("runtime.fit_parameters")?;
        let rows = fit_observations::Row::rows(&observations)?;
        assert_eq!(rows.len(),1);
        close_to(rows[0].prediction.unwrap(),2.,1e-10);
        close_to(rows[0].objective_contribution.unwrap(),0.,1e-20);
        // An explicit native backend is a meaningful incompatible LM request;
        // preparation must refuse before changing the requested optimizer.
        let error = prepare_measurement(&context,&package,
            measurement_profile(FitOptimizer::ResidualLm,Some(Backend::Ipopt))?,&cancel).await.unwrap_err();
        assert!(error.to_string().contains("residual LM cannot implement the requested native-only controls"),"{error}");
        drop((result,handle,prepared,package));
        Ok((observations,parameters))
    }).catch_unwind().await;
    let (observations,parameters) = finish(outcome,context.close().await)?;
    assert_eq!(fit_observations::Row::rows(&observations)?.len(),1);
    let parameters = fit_parameters::Row::rows(&parameters)?;
    assert_eq!(parameters.len(),1);
    assert!(!parameters[0].fixed);
    close_to(parameters[0].value.unwrap(),2.,1e-10);
    Ok(())
}

#[tokio::test]
async fn native_all_fixed_fit_evaluates_original_sources_through_shared_completion_and_owned_tables() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = measurement_fit(&context,true).await?;
        let cancel = CancelSource::new();
        let prepared = prepare_measurement(&context,&package,
            measurement_profile(FitOptimizer::Automatic,None)?,&cancel).await?;
        assert_eq!(prepared.route(),FitRoute::Evaluation);
        assert_eq!(prepared.composition_request().policy,pse_model::strategy::CompositionPolicy::Auto);
        let original_identity = prepared.identity();
        let profile_identity = prepared.strategy_profile();
        assert!(!profile_identity.to_prefixed().is_empty());
        let strategy = pse_model::strategy::NumericalStrategyDocument::new(prepared.numerical_strategy())?;
        assert_eq!(serde_json::to_value(&strategy)?["version"],4);
        assert_eq!(strategy.strategy.mechanisms.len(),1);
        assert!(!strategy.strategy.mechanisms[0].support.is_empty());
        let decoded: pse_model::strategy::NumericalStrategyDocument = serde_json::from_slice(&serde_json::to_vec(&strategy)?)?;
        assert_eq!(decoded,strategy);
        let sources = Box::pin(package.source_tables(&context.runtime.preparation_services())).await?;
        let fits = fit_cases::Row::rows(&sources[&fit_cases::RELATION_ID])?;
        assert_eq!(fits.len(),1);
        assert_eq!(&fits[0],prepared.declaration());
        assert_eq!(fits[0].fit_id,SemanticId::from_bytes([158;16]).into());
        assert_eq!(fits[0].parameters.len(),1);
        assert!(fits[0].parameters[0].fixed);
        assert_eq!(fits[0].parameters[0].value,2.);
        assert_eq!((fits[0].parameters[0].lower,fits[0].parameters[0].upper),(Some(0.),Some(10.)));
        let source_fit = sources[&fit_cases::RELATION_ID].clone();
        let handle = prepared.start(&context.runtime.execution_services())?;
        let result = handle.wait().await?;
        assert_eq!(handle.wait().await?.run_id,result.run_id);
        let RunRequest::Fit(request) = result.request() else {panic!("original all-fixed fit request")};
        assert_eq!(request.identity(),original_identity);
        assert_eq!(request.strategy_profile(),profile_identity);
        assert_eq!(request.route(),FitRoute::Evaluation);
        let RunReport::Fit(report) = result.report().unwrap() else {panic!("original all-fixed fit report")};
        assert!(matches!(report.execution_document(),FitExecutionDocument::Evaluation));
        assert!(matches!(report.profile_document(),FitProfileDocument::NotRequested));
        assert!(report.solve.is_none() && report.residual.is_none(),"all-fixed evaluation runs no optimizer");
        assert!(report.candidate.as_ref().is_some_and(Vec::is_empty),
            "all-fixed evaluation has no free optimization coordinates");
        assert_eq!(report.objective,Some(0.));
        assert!(result.usable() && result.completion().unwrap().diagnostics.is_empty());
        assert!(result.tables()?.contains_key(&fit_observations::RELATION_ID));
        let observations = result.table("runtime.fit_observations")?;
        let checks = result.table("runtime.modeling_checks")?;
        let events = result.table("runtime.solve_strategy_events")?;
        let observed = fit_observations::Row::rows(&observations)?;
        assert_eq!(observed.len(),1);
        assert_eq!(observed[0].prediction,Some(2.));
        assert_eq!(observed[0].objective_contribution,Some(0.));
        let parameters = fit_parameters::Row::rows(&result.table("runtime.fit_parameters")?)?;
        assert_eq!(parameters.len(),1);
        assert!(parameters[0].fixed);
        assert_eq!(parameters[0].value,Some(2.));
        let checks_rows = modeling_checks::Row::rows(&checks)?;
        assert!(!checks_rows.is_empty() && checks_rows.iter().all(|check|check.satisfied));
        let event_rows = solve_strategy_events::Row::rows(&events)?;
        assert!(event_rows.iter().all(|event|event.decision_identity.is_some()));
        let last = event_rows.last().expect("shared fit finish event");
        assert_eq!(last.transition,Some(pse_model::generated::enums::NumericalTransition::Finish));
        assert_eq!(last.evaluations,None,"evaluation-only completion invents no native work counts");
        let mut unsupported = measurement_profile(FitOptimizer::Automatic,None)?;
        unsupported.solver.presolve = pse_backend_native::presolve::Policy::from_native_options(
            &Default::default(),[pse_backend_native::presolve::Pass::AffineElimination].into())?;
        let error = prepare_measurement(&context,&package,unsupported,&cancel).await.unwrap_err();
        assert!(error.to_string().contains("all-fixed fitting evaluates directly and cannot apply required native presolve passes"),
            "{error}");
        drop((result,handle,prepared,package));
        Ok((observations,checks,events,source_fit))
    }).catch_unwind().await;
    let (observations,checks,events,source_fit) = finish(outcome,context.close().await)?;
    assert_eq!(fit_observations::Row::rows(&observations)?[0].prediction,Some(2.));
    assert!(modeling_checks::Row::rows(&checks)?.iter().all(|check|check.satisfied));
    assert!(!solve_strategy_events::Row::rows(&events)?.is_empty());
    assert!(fit_cases::Row::rows(&source_fit)?[0].parameters[0].fixed);
    Ok(())
}

async fn analysis(
    context: &NativeContext,
    package: &ModelingPackage,
    settings: SolveSettings,
) -> Result<ModelingAnalysis> {
    let root = package.declarations(&context.runtime.preparation_services()).await?
        .iter().find(|row| row.name == "Root").unwrap().declaration_id;
    Ok(ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Default::default(),
        limits: Default::default(),
        case: Default::default(),
        order: pse_kernels::DerivativeOrder::Second,
        compiler: Default::default(),
        solver: settings.profile()?,
        numerical: Default::default(),
    })
}

#[tokio::test]
async fn native_smooth_optimization_checks_objective_and_retains_stopped_candidate() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        // Nonlinear equality and quartic objective: the unique zero-cost feasible
        // point is x=2, y=4, independently of any native solver verdict.
        let package = context.package(
            "package science { def Root { var x:Scalar; var y:Scalar;
             eq curve:y==x*x; let cost:Scalar=(x-2)*(x-2)+(y-4)*(y-4);
             annotation objective cost(minimize);
             annotation bounds x(0,10); annotation bounds y(0,100);
             annotation start x(0.5); annotation start y(0.5); }}"
        ).await?;
        let services = context.runtime.execution_services();
        let cancel = CancelSource::new();
        let settings = SolveSettings {
            backend: Some(Backend::Ipopt), intent: SolveIntent::Optimize,
            ..Default::default()
        };
        let mut request = analysis(&context, &package, settings.clone()).await?;
        request.bindings.demand = ["x", "y"].map(String::from).to_vec();
        let prepared = Box::pin(package.prepare_analysis(services.preparation(), &request, &cancel)).await?;
        let paths = &prepared.model.model.compiled().model.paths;
        let (x_id, y_id) = (paths["x"], paths["y"]);
        // Request physical accuracy at the actual admitted targets. Default
        // engineering budgets do not promise this tighter analytic comparison.
        request.solver.numerics.requirements = prepared.solve.numerics().targets.iter().map(|target|
            pse_model::numerics::NumericalRequirement {
                requirement_id: pse_ids::named_id(request.root.as_id(), &format!("native-nlp:{:?}:{}", target.kind, target.id)),
                model_id: None, case_id: None, instance_id: None, fit_id: None,
                target_id: target.id, target_kind: target.kind,
                nominal: None, scaling_factor: None, absolute_tolerance: Some(1e-10),
                relative_tolerance: Some(0.), shared_engineering_allowance: None,
                engineering_rule_id: None, unit_id: Some(target.unit),
                coordinates: pse_model::generated::enums::NumericalCoordinates::Physical,
                priority: 0, required: true, provenance: "native nonlinear optimum oracle".into(),
            }
        ).collect();
        request.solver.numerics.engineering_relative_fraction = 1e-10;
        request.solver.numerics.kkt = pse_model::numerics::KktTolerances {
            stationarity: 1e-10, complementarity: 1e-10,
        };
        drop(prepared);
        let prepared = Box::pin(package.prepare_analysis(services.preparation(), &request, &cancel)).await?;
        let result = prepared.start(&services)?.wait().await?;
        assert!(result.usable(), "{:?}", result.report());
        let variables = solve_variables::Row::rows(&result.table("runtime.solve_variables")?)?;
        let value = |id| variables.iter().find(|row| row.symbol_id == id).unwrap().value.unwrap();
        let (x, y) = (value(x_id), value(y_id));
        close_to(x, 2., 1e-5);
        close_to(y, 4., 1e-5);
        close_to(y, x*x, 1e-6);
        let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs")?)?;
        let run = runs.last().unwrap();
        close_to(run.objective.unwrap(), (x-2.).powi(2)+(y-4.).powi(2), 1e-8);
        close_to(run.objective.unwrap(), 0., 1e-8);

        let mut stopped = settings;
        stopped.controls.iterations = 1;
        let request = analysis(&context, &package, stopped).await?;
        let prepared = Box::pin(package.prepare_analysis(services.preparation(), &request, &cancel)).await?;
        let stopped = prepared.start(&services)?.wait().await?;
        assert!(!stopped.usable(), "one nonconverged step must not qualify: {:?}", stopped.report());
        assert!(!solve_runs::Row::rows(&stopped.table("runtime.solve_runs")?)?.is_empty(),
            "the stopped native attempt remains inspectable");
        Ok(())
    }).catch_unwind().await;
    finish(outcome, context.close().await)
}

#[tokio::test]
async fn native_weighted_fit_checks_inverse_information_and_refuses_zero_sigma() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let case = covariance::LineCase::weighted();
        let report = covariance::run(&context, &case).await?;
        assert!(report.complete);
        // Direct normal equations for absolute known sigma: estimates are
        // (X'WX)^-1 X'Wy and covariance is (X'WX)^-1, with no RSS rescaling.
        let weight = 1. / case.sigma.powi(2);
        let n = case.x.len() as f64;
        let sx: f64 = case.x.iter().sum();
        let sxx: f64 = case.x.iter().map(|x| x*x).sum();
        let sy: f64 = case.y.iter().sum();
        let sxy: f64 = case.x.iter().zip(&case.y).map(|(x,y)| x*y).sum();
        let determinant = n*sxx-sx*sx;
        let expected = BTreeMap::from([
            ("a", (sxx*sy-sx*sxy)/determinant),
            ("b", (n*sxy-sx*sy)/determinant),
        ]);
        for (name, expected) in expected {
            close_to(report.estimates[name], expected, report.estimate_allowances[name]);
        }
        let covariance = &report.covariance;
        assert_eq!(covariance.coordinates, "physical");
        assert_eq!(covariance.convention, "inverse_information_known_sigma");
        for (i, left) in covariance.parameters.iter().enumerate() {
            for (j, right) in covariance.parameters.iter().enumerate() {
                let numerator = match (left.as_str(), right.as_str()) {
                    ("a", "a") => sxx,
                    ("b", "b") => n,
                    ("a", "b") | ("b", "a") => -sx,
                    _ => panic!("unexpected fitted parameter"),
                };
                close_to(covariance.values[2*i+j], numerator/(weight*determinant), 1e-8);
            }
        }
        let mut invalid = case;
        invalid.sigma = 0.;
        let error = match covariance::run(&context, &invalid).await {
            Err(error) => error,
            Ok(_) => panic!("zero-sigma included observations must refuse preparation"),
        };
        assert!(error.to_string().contains("positive difference-unit standard deviations"), "{error}");
        Ok(())
    }).catch_unwind().await;
    finish(outcome, context.close().await)
}

#[tokio::test]
async fn native_sensitivity_checks_active_branch_and_refuses_unknown_parameter() -> Result<()> {
    let native = sensitivity::NativeCandidate::new()?;
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let case = sensitivity::SensitivityCase::quadratic();
        let report = native.run(&case).await?;
        assert!(report.complete);
        // Stationarity under x+y=a gives x=(2a+b)/3 and y=(a-b)/3.
        // Strictly positive a keeps z=0 on its active lower bound.
        for (name, derivative) in [("x", [2./3., 1./3.]), ("y", [1./3., -1./3.]), ("z", [0., 0.])] {
            let q = &report.quantities[name];
            let nominal = match name {
                "x" => (2.*case.nominal["a"]+case.nominal["b"])/3.,
                "y" => (case.nominal["a"]-case.nominal["b"])/3.,
                _ => 0.,
            };
            let perturbed = match name {
                "x" => (2.*case.perturbed["a"]+case.perturbed["b"])/3.,
                "y" => (case.perturbed["a"]-case.perturbed["b"])/3.,
                _ => 0.,
            };
            close_to(q.base, nominal, q.allowance);
            close_to(q.predicted, perturbed, q.allowance);
            for (parameter, expected) in ["a", "b"].into_iter().zip(derivative) {
                close_to(q.derivative[parameter], expected, q.allowance);
            }
        }
        let package = context.package(
            "package science { def Root { param a:Scalar=1; var x:Scalar;
             let cost:Scalar=(x-a)*(x-a); annotation objective cost(minimize);
             annotation start x(0); }}"
        ).await?;
        let request = analysis(&context, &package, SolveSettings {
            backend: Some(Backend::Ipopt), intent: SolveIntent::Optimize,
            sensitivity: Some(SensitivityRequest {
                parameters: vec![SemanticId::from_bytes([239;16])],
                reduced_hessian: false, propagation: None,
            }),
            ..Default::default()
        }).await?;
        let error = match Box::pin(package.prepare_analysis(
            &context.runtime.preparation_services(), &request, &CancelSource::new()
        )).await {
            Err(error) => error,
            Ok(_) => panic!("an unknown parameter must refuse before native execution"),
        };
        assert!(error.to_string().contains("not a declared fixed parameter"), "{error}");
        Ok(())
    }).catch_unwind().await;
    // Both independent owners settle, even if either close fails.
    let (native_closed, context_closed) = tokio::join!(native.close(), context.close());
    finish(outcome, native_closed.and(context_closed))
}

#[tokio::test]
async fn native_degeneracy_checks_nullspace_and_retains_attempt_limited_evidence() -> Result<()> {
    let native = degeneracy::NativeCandidate::new()?;
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let case = degeneracy::LinearCase::dependent();
        let report = native.run(&case).await?;
        assert!(report.complete, "{:?}", report.unavailable);
        assert!(report.unavailable.is_empty());
        let actual: BTreeSet<_> = report.sets.iter().map(|set|
            set.rows.iter().map(String::as_str).collect::<BTreeSet<_>>()
        ).collect();
        assert_eq!(actual, BTreeSet::from([
            BTreeSet::from(["first", "second"]),
            BTreeSet::from(["third", "fourth", "fifth"]),
        ]));
        for set in &report.sets {
            assert!(set.irreducible_at_tolerance);
            assert!(set.multipliers.values().any(|weight| weight.abs() > 1e-6));
            // Independent certificate: weighted original coefficients AND RHS
            // vanish, so published scaling cannot hide an incorrect multiplier.
            for column in 0..=case.variables.len() {
                let residual: f64 = case.rows.iter().map(|row| {
                    set.multipliers.get(&row.name).copied().unwrap_or(0.) *
                        if column == case.variables.len() { row.rhs } else { row.coefficients[column] }
                }).sum();
                close_to(residual, 0., 1e-6);
            }
        }
        let package = context.package(
            "package science { def Root { var x:Scalar; var y:Scalar;
             eq first:x+y==1; eq second:2*x+2*y==2;
             annotation start x(0.5); annotation start y(0.5); }}"
        ).await?;
        let services = context.runtime.execution_services();
        let cancel = CancelSource::new();
        let request = analysis(&context, &package, SolveSettings {
            intent: SolveIntent::Root, ..Default::default()
        }).await?;
        let prepared = Box::pin(package.prepare_diagnostics(services.preparation(), &request, &cancel)).await?;
        let values = prepared.model.values.clone();
        let partial = Box::pin(package.diagnose_jacobian_optimization(
            &services, prepared, values,
            JacobianDiagnosticControls { maximum_attempts: 1, ..Default::default() }.policy(),
            Default::default(),
            &DiagnosticOperation::new(&cancel, Duration::from_secs(120), None)?,
        )).await?;
        assert!(!partial.evidence.complete);
        assert_eq!(partial.evidence.attempts.len(), 1);
        assert!(!partial.evidence.conditioning.is_empty(), "retain the completed LP certificate");
        assert!(partial.evidence.degenerate.is_empty(), "unvisited MILP anchors cannot supply complete sets");
        assert!(partial.evidence.unavailable.iter().any(|reason| reason.contains("attempt budget")));
        Ok(())
    }).catch_unwind().await;
    let (native_closed, context_closed) = tokio::join!(native.close(), context.close());
    finish(outcome, native_closed.and(context_closed))
}
