// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compact scientific fixtures; case declarations select shape, never a second runner.
use super::*;
use pse_backend_native::{self as native, dynamics, solve::SolveIntent};
use pse_ids::{SemanticId, named_id};
use pse_kernels::DerivativeOrder;
use pse_model::generated::enums::NumericalTarget;
use serde_json::Value;
use std::sync::Arc;

fn id(name: &str) -> SemanticId {
    named_id(SemanticId::NIL, &format!("process-cost.compact.{name}"))
}

async fn package(owner: &WorkflowRuntime, source: &str) -> (ModelingPackage, DeclarationId) {
    let rows = pse_authoring::language::parse(
        source, id("package"), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = rows.iter().find(|row| row.name == "Root").unwrap().declaration_id;
    let package = runtime(owner).modeling_package(rows, physical(owner).await).await.unwrap();
    (package, root)
}

/// Analytic diagonal/banded/coupled roots, with a fixed original solution x_i=i+1.
async fn roots(owner: &WorkflowRuntime, spec: &Value, observations: &mut observations::Observations) {
    let n = spec["blocks"].as_u64().unwrap() as usize;
    assert!(n > 0);
    let shape = spec["shape"].as_str().unwrap_or("diagonal");
    assert!(matches!(shape, "diagonal" | "banded" | "coupled"));
    let mut source = String::from("package compact { def Root {");
    for i in 0..n {
        source.push_str(&format!("var x{i}:Scalar; annotation start x{i}(0.5);"));
    }
    for i in 0..n {
        let extra = match shape {
            "diagonal" => String::new(),
            "banded" if i > 0 => format!("+0.1*(x{}-{})", i-1, i),
            "banded" => String::new(),
            "coupled" => (0..n).filter(|j| *j != i).map(|j| format!("+0.1*(x{j}-{})", j+1)).collect(),
            _ => unreachable!(),
        };
        source.push_str(&format!(
            "eq row{i}:x{i}*x{i}{extra}=={}; annotation bounds x{i}(0.1,{}); annotation check x{i}(abs(x{i}-{})<=0.005);",
            (i+1)*(i+1), n+2, i+1,
        ));
    }
    source.push_str("} }");
    let (package, root) = package(owner, &source).await;
    let backend = match spec["backend"].as_str().unwrap_or("kinsol") {
        "kinsol" => Backend::Kinsol, "petsc" => Backend::Petsc,
        value => panic!("unsupported compact root backend {value}"),
    };
    let mut selected = profile(backend, false);
    selected.intent = SolveIntent::Root;
    selected.presolve = native::presolve::Policy::Off;
    if backend == Backend::Petsc { selected.numerics.native_scaling = false; }
    let cancel = audit::cancel_source();
    let analysis = package.declared_execution(
        root, compiler(), selected, Default::default(), seed_limits(), &cancel,
    ).await.unwrap().analysis;
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let symbols = (0..n).map(|i| {
        prepared.model.model.compiled().model.symbols.values()
            .find(|symbol| symbol.lineage.path.ends_with(&format!(".x{i}"))).unwrap().id
    }).collect::<Vec<_>>();
    let result = audit::wait(prepared.start().unwrap()).await.unwrap();
    let report = authored_success(&result);
    let pse_runtime::math::solves::Outcome::Native(native) = &report.outcome else {
        panic!("expected native root");
    };
    assert_eq!(native.backend, backend);
    for (i, symbol) in symbols.into_iter().enumerate() {
        near(variable(&result, symbol), (i+1) as f64, 0.005);
    }
    observations.run(&result, owner.runtime.math());
}

/// Checked LP/MILP commitment and bounded synthesis MINLP owner templates.
async fn classes(owner: &WorkflowRuntime, operation: &str, observations: &mut observations::Observations) {
    let (source, expected, backend) = match operation {
        "compact-lp" => (
            "package compact { def Root { var x:Power; var y:Power; eq total:x+y<=3.5{W}; eq cap:x<=0.4{W}; let cost:Power=-x-2*y; annotation objective cost(minimize); annotation bounds x(0{W},10{W}); annotation bounds y(0{W},5{W}); annotation start x(0{W}); annotation start y(0{W}); annotation check cost(abs(cost+7{W})<=0.01{W}); annotation report cost(\"cost\"); } }",
            -7.0, Backend::Highs,
        ),
        "compact-milp" => (
            "package compact { def Root { param unit:Power=1{W}; var x:Power; var y:Count in integer; eq total:x+unit*y<=3.5{W}; eq cap:x<=0.4{W}; let cost:Power=-x-2*unit*y; annotation objective cost(minimize); annotation bounds x(0{W},10{W}); annotation bounds y(0{1},5{1}); annotation start x(0{W}); annotation start y(0{1}); annotation check cost(abs(cost+6.4{W})<=0.01{W}); annotation report cost(\"cost\"); } }",
            -6.4, Backend::Highs,
        ),
        "compact-minlp" => (
            "package compact { def Root { param unit:Power=1{W}; param fee:Power=2{W}; var x:Scalar; var s:Scalar; var build:Indicator in binary; eq demand:x*build+s>=3; let cost:Power=fee*build+unit*(0.5*x*x+4*s); annotation objective cost(minimize); annotation bounds x(0,4); annotation bounds s(0,3); annotation start x(0); annotation start s(3); annotation start build(0{1}); annotation report cost(\"cost\"); annotation check cost(abs(cost-6.5{W})<=0.01{W}); } }",
            6.5, Backend::Scip,
        ),
        "compact-uno" => (
            "package compact { def Root { var x:Scalar; let cost:Scalar=x*x; annotation objective cost(minimize); annotation bounds x(1,2); annotation start x(1.5); annotation report cost(\"cost\"); annotation check x(abs(x-1)<=0.005); } }",
            1.0, Backend::Uno,
        ),
        _ => panic!("unknown compact class {operation}"),
    };
    let (package, root) = package(owner, source).await;
    let cancel = audit::cancel_source();
    let mut selected = profile(backend, true);
    selected.presolve = native::presolve::Policy::Off;
    if backend == Backend::Uno { selected.controls.hessian = native::solve::HessianMode::LimitedMemory; }
    let analysis = package.declared_execution(
        root, compiler(), selected, Default::default(), seed_limits(), &cancel,
    ).await.unwrap().analysis;
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let result = audit::wait(prepared.start().unwrap()).await.unwrap();
    let report = authored_success(&result);
    let pse_runtime::math::solves::Outcome::Native(native) = &report.outcome else {
        panic!("expected native class result");
    };
    assert_eq!(native.backend, backend);
    near(report.reports.iter().find(|report| report.label == "cost").unwrap().value, expected, 0.01);
    assert!(native.quality.as_ref().unwrap().feasible());
    if operation == "compact-milp" || operation == "compact-minlp" {
        assert_eq!(native.quality.as_ref().unwrap().integrality.len(), 1);
    }
    if operation == "compact-minlp" {
        assert_eq!(native.qualification, native::solve::Qualification::GapQualified);
        assert_eq!(native.termination.assurance, native::solve::Assurance::GlobalBound);
        near(native.evidence.global.as_ref().unwrap().dual_bound.unwrap(), 6.5, 0.01);
    }
    observations.run(&result, owner.runtime.math());
}

/// ODE and index-1 DAE from modeling/dynamics.rs; physical time is two seconds.
async fn trajectory(owner: &WorkflowRuntime, spec: &Value, observations: &mut observations::Observations) {
    let method = match spec["backend"].as_str().unwrap_or("diffsol") {
        "diffsol" => dynamics::Method::Diffsol, "idas" => dynamics::Method::Idas,
        backend => panic!("unknown compact dynamics backend {backend}"),
    };
    let dae = spec["shape"].as_str() == Some("dae");
    let scheduled = spec["scheduled"].as_bool().unwrap_or(false);
    let count = spec["samples"].as_u64().unwrap_or(5) as usize;
    assert!(count >= 2);
    let equation = if dae { "d(x[i])/di == y" } else { "d(x[i])/di == p" };
    let algebraic = if dae { "var y:Scalar; eq closure:y==2*p; annotation start y(4);" } else { "" };
    let source = format!(
        "package compact {{ def Root {{ domain t:Time from 0{{s}} to 2{{s}}; discretize grid on t using integrated(elements=1,order=1); param p:Scalar=2; param offset:Time=1{{s}}; var x[i in t]:Time; eq rate[i in t]:{equation}; eq initial:x[0{{s}}]==offset; {algebraic} annotation start x(1{{s}}); annotation check x(x[i]>=0{{s}} and x[i]<=10{{s}}); }} }}"
    );
    let (package, root) = package(owner, &source).await;
    let cancel = audit::cancel_source();
    let accuracy = pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
    let integration = dynamics::Profile {
        method, end: 2.0, samples: (0..count).map(|i| 2.0*i as f64/(count-1) as f64).collect(),
        rtol: accuracy, atol: vec![accuracy; if dae {2} else {1}],
        sensitivity: dynamics::DynamicSensitivity::Forward,
        parameter_scales: vec![1.0, 1.0],
        ..Default::default()
    };
    let initial = package.prepare_simulation(
        root, pse_modeling::specialize::root_instance(root), Default::default(), seed_limits(),
        Default::default(), compiler(), integration, DerivativeOrder::First, &cancel,
    ).await.unwrap();
    let p_id = initial.model().model.compiled().model.symbols.values()
        .find(|symbol| symbol.lineage.path.ends_with(".p")).unwrap().id;
    let p = initial.contract().parameters.iter().position(|parameter| *parameter == p_id).unwrap();
    let prepared = if scheduled {
        let mut integration = initial.profile().clone();
        integration.schedule = vec![dynamics::ScheduledInput {parameter: p, times: vec![1.0]}];
        package.prepare_simulation(
            root, pse_modeling::specialize::root_instance(root), Default::default(), seed_limits(),
            Default::default(), compiler(), integration, DerivativeOrder::First, &cancel,
        ).await.unwrap()
    } else { initial };
    let contract = prepared.contract();
    let xi = contract.differential.iter().position(|differential| *differential).unwrap();
    let x = contract.states[xi];
    let allowance = resolved_allowance(prepared.numerics(), NumericalTarget::Variable, x);
    let result = audit::wait(prepared.start().unwrap()).await.unwrap();
    assert!(result.usable(), "{:?}", result.assessments());
    let RunReport::Simulation(trajectory) = result.report().unwrap() else {panic!("missing trajectory")};
    assert!(trajectory.accepted(), "{trajectory:?}");
    let report = trajectory.report();
    assert_eq!(report.termination, dynamics::Termination::Completed);
    assert_eq!(report.samples.len(), count);
    for sample in &report.samples {
        // The scheduled second interval retains p=2; it deliberately exercises
        // restart without changing the independent physical solution.
        near(sample.outputs[xi], 1.0 + (if dae {4.0} else {2.0})*sample.time, allowance);
        if dae { near(sample.outputs[1-xi], 4.0, resolved_allowance(prepared.numerics(), NumericalTarget::Variable, contract.states[1-xi])); }
    }
    assert!(report.error.is_none(), "{:?}", report.error);
    observations.trajectory(report);
}

/// The checked one-definition steady/integrated/simultaneous template in dynamics.rs.
/// Four backward-Euler elements are a discretization control, not an ODE accuracy claim.
async fn decay(owner: &WorkflowRuntime, spec: &Value, observations: &mut observations::Observations) {
    let tau = if spec["stiff"].as_bool().unwrap_or(false) {0.01} else {1.0};
    let source = format!(
        "package compact {{ difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0.5,0.5); def Root {{ domain t:Time from 0{{s}} to 1{{s}}; when analysis.route==analysis.integrated {{discretize native on t using integrated(elements=1,order=1);}} when analysis.route!=analysis.integrated {{discretize grid on t using backward(elements=4,order=1);}} param target:Time=2{{s}}; param tau:Time={tau}{{s}}; var x[i in t]:Time; when analysis.dynamic {{eq transient[i in t]:d(x[i])/di==(target-x[i])/tau; eq initial:x[0{{s}}]==0{{s}};}} when not analysis.dynamic {{eq stationary[i in t]:0==(target-x[i])/tau;}} annotation start x(0{{s}}); annotation check x(x[i]>=-0.01{{s}} and x[i]<=2.01{{s}}); }} }}"
    );
    let (package, root) = package(owner, &source).await;
    let cancel = audit::cancel_source();
    if spec["operation"].as_str() == Some("compact-simultaneous") {
        let mut selected = profile(Backend::Kinsol, false);
        selected.presolve = native::presolve::Policy::Off;
        let prepared = package.prepare_solve(
            root, pse_modeling::specialize::root_instance(root),
            pse_modeling::Bindings::default().with_analysis(pse_modeling::analysis::Route::Simultaneous),
            seed_limits(), Default::default(), DerivativeOrder::First, compiler(), selected,
            Default::default(), &cancel,
        ).await.unwrap();
        let variables = prepared.model.model.compiled().admitted.case().variables().iter()
            .map(|variable| variable.port.id).collect::<Vec<_>>();
        let result = audit::wait(prepared.start().unwrap()).await.unwrap();
        authored_success(&result);
        let end = variables.into_iter().map(|symbol| variable(&result,symbol)).fold(f64::NEG_INFINITY,f64::max);
        near(end,2.0*(1.0-(1.0+0.25/tau).powi(-4)),0.005);
        observations.run(&result, owner.runtime.math());
    } else {
        let method = match spec["backend"].as_str().unwrap_or("diffsol") {
            "diffsol" => dynamics::Method::Diffsol, "idas" => dynamics::Method::Idas,
            backend => panic!("unsupported decay backend {backend}"),
        };
        let count = spec["samples"].as_u64().unwrap_or(5) as usize;
        assert!(count >= 2);
        let accuracy = pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
        let integration = dynamics::Profile {
            method, end:1.0, samples:(0..count).map(|i|i as f64/(count-1) as f64).collect(),
            rtol:accuracy, atol:vec![accuracy], parameter_scales:vec![1.0,1.0],
            ..Default::default()
        };
        let prepared = package.prepare_simulation(
            root, pse_modeling::specialize::root_instance(root), Default::default(), seed_limits(),
            Default::default(), compiler(), integration, DerivativeOrder::First, &cancel,
        ).await.unwrap();
        let budget = resolved_allowance(prepared.numerics(), NumericalTarget::Variable, prepared.contract().states[0]);
        let result = audit::wait(prepared.start().unwrap()).await.unwrap();
        assert!(result.usable(),"{:?}",result.assessments());
        let RunReport::Simulation(trajectory) = result.report().unwrap() else {panic!("missing decay trajectory")};
        assert!(trajectory.accepted(),"{trajectory:?}");
        assert_eq!(trajectory.report().samples.len(),count);
        for sample in &trajectory.report().samples {
            near(sample.outputs[0],2.0*(1.0-(-sample.time/tau).exp()),budget);
        }
        observations.trajectory(trajectory.report());
    }
}

async fn reference(owner: &WorkflowRuntime, operation: &str, observations: &mut observations::Observations) {
    let loading = audit::stage("reference_documents_and_admission");
    let package = seed_package(owner).await;
    assert!(package.declarations().await.unwrap().iter().any(|row| row.name == "nitrogen_shomate_values"));
    drop(loading);
    if operation == "compact-reference" { std::hint::black_box(package); return; }
    let case = SemanticId::parse_hex("01a0e1484fb87705b3f22bd8fe49aa48").unwrap();
    let prepared = seed_prepare(&package, case, profile(Backend::Kinsol, false), &audit::cancel_source()).await.unwrap();
    let result = audit::wait(prepared.start().unwrap()).await.unwrap();
    let report = authored_success(&result);
    assert!(report.checks.len() >= 4);
    observations.run(&result, owner.runtime.math());
}

/// Affirmative IFT derivative mechanism at an independently known original root.
/// This is the public math boundary, not a claim about multivariate branch selection
/// or inner nonlinear iteration. The scalar authored nested case is separate.
fn implicit_derivatives(spec: &Value) {
    use pse_math::{implicit, typed::{Binary, BodyBuilder, BodyLimits}};
    let n = spec["blocks"].as_u64().unwrap() as usize;
    assert!(n > 0);
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = registry.neutral_dimensionless().unwrap();
    let mut builder = BodyBuilder::new(
        pse_math::initialize().unwrap(), &registry,
        &pse_quantity::standard::StandardInvariantChecker, n+2, BodyLimits::default(),
    ).unwrap();
    let inputs = (0..n+2).map(|slot| builder.input(slot, quantity, Default::default(), id("residual")).unwrap()).collect::<Vec<_>>();
    let square_p = builder.binary(Binary::Mul, inputs[n].clone(), inputs[n].clone(), None, id("residual")).unwrap();
    let cross = builder.binary(Binary::Mul, inputs[n].clone(), inputs[n+1].clone(), None, id("residual")).unwrap();
    let square_q = builder.binary(Binary::Mul, inputs[n+1].clone(), inputs[n+1].clone(), None, id("residual")).unwrap();
    let mut residuals = Vec::new();
    for input in &inputs[..n] {
        let residual = builder.binary(Binary::Sub, input.clone(), square_p.clone(), None, id("residual")).unwrap();
        let residual = builder.binary(Binary::Sub, residual, cross.clone(), None, id("residual")).unwrap();
        residuals.push(builder.binary(Binary::Sub, residual, square_q.clone(), None, id("residual")).unwrap());
    }
    let control = pse_diagnostics::Cancellation::new(false);
    let _binding = audit::bind_math_control(&control);
    let scope = pse_kernels::ExecutionScope::new(control.clone(), None);
    let parameter = Arc::new(builder.finish(
        &residuals, DerivativeOrder::Second, pse_math::library::Optimization::default(), &scope,
    ).unwrap());
    let unknown = Arc::new(parameter.prepared_support().incidence(&(0..n).collect::<Vec<_>>(), &control.flag()).unwrap()
        .at_order(DerivativeOrder::First).unwrap().compile(
            pse_math::library::Optimization::default(), pse_math::jets::EvaluationLimits::default(), &scope,
        ).unwrap());
    let problem = implicit::Problem::new(
        id("implicit"), pse_ids::ContentHash::from_bytes([0;32]),
        (0..n).map(|i| implicit::Unknown {id:id(&format!("unknown.{i}")), lower:0.0, upper:50.0}).collect(),
        (0..n).map(|i| id(&format!("row.{i}"))).collect(), 2,
        implicit::ResidualBodies {unknown, parameter}, 100_000,
    ).unwrap();
    let options = implicit::Options {
        start:vec![19.0;n], variable_nominals:vec![20.0;n], variable_tolerance:vec![0.02;n],
        residual_tolerance:vec![0.001;n], iterations:30, time_limit:Duration::from_secs(1),
        derivative_tolerance:pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction,
    };
    let order = match spec["derivative_order"].as_u64().unwrap_or(2) {
        1 => DerivativeOrder::First, 2 => DerivativeOrder::Second,
        order => panic!("unsupported implicit derivative order {order}"),
    };
    let jet = problem.derivatives(&[2.0,3.0], &vec![19.0;n], order, &options, &control.flag()).unwrap();
    assert_eq!(jet.values, vec![19.0;n]);
    for row in 0..n {
        assert_eq!(&jet.jacobian[row*2..row*2+2], &[7.0,8.0]);
        if order == DerivativeOrder::Second {assert_eq!(&jet.hessians[row*4..row*4+4], &[2.0,1.0,1.0,2.0]);}
    }
    audit::checkpoint();
    std::hint::black_box(jet);
}

async fn implicit_root(owner: &WorkflowRuntime, observations: &mut observations::Observations) {
    let (package, root) = package(owner,
        "package compact { def Root { var x:Scalar; implicit inner select branch(y>=0) {var y:Scalar; eq root:y*y==x; annotation bounds y(0.5,3); annotation start y(1);} realize policy on inner using nested; eq pin:inner.y==2; annotation bounds x(0.3,8); annotation start x(3); annotation report x(\"parameter\"); annotation check x(abs(x-4)<=0.01); } }"
    ).await;
    let cancel = audit::cancel_source();
    let mut selected = profile(Backend::Kinsol, false);
    selected.presolve = native::presolve::Policy::Off;
    let analysis = package.declared_execution(root, compiler(), selected, Default::default(), seed_limits(), &cancel).await.unwrap().analysis;
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let result = audit::wait(prepared.start().unwrap()).await.unwrap();
    let report = authored_success(&result);
    near(report.reports.iter().find(|report| report.label=="parameter").unwrap().value,4.0,0.01);
    observations.run(&result, owner.runtime.math());
}

/// Four actual points from the lane-current scientific control in study_execution.rs.
/// Full and Compact retention preserve the same original values and checks.
async fn study(owner: &WorkflowRuntime, spec: &Value, observations: &mut observations::Observations) {
    use pse_model::{generated::enums::StudyPointState, scalars::FiniteBound, study::*};
    use pse_relations::{columnar::RelationRow, generated::runtime::{modeling_checks, solve_variables, solve_metrics, solve_strategy_events}};
    use pse_runtime::workflow;
    let workers = spec["workers"].as_u64().unwrap() as usize;
    assert!(matches!(workers, 1 | 2));
    assert_eq!(spec["blocks"].as_u64(), Some(4));
    assert_eq!(owner.runtime.budget().threads.pool_threads.get(), workers);
    let continuation = spec["continuation"].as_bool().unwrap_or(false);
    let compact = match spec["retention"].as_str().unwrap_or("full") {
        "full" => false, "compact" => true,
        retention => panic!("unsupported compact study retention {retention}"),
    };
    let physical = physical(owner).await;
    let scalar = physical.quantities().neutral_dimensionless().unwrap();
    let unit = physical.quantities().quantity_type(scalar).unwrap().canonical_unit.as_id();
    let rows = pse_authoring::language::parse(
        "package compact { def Root { param t:Scalar=1; var x:Scalar; eq e:x==2+t; annotation start x(2+t); annotation check x(x>0); } }",
        id("study-package"), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = rows.iter().find(|row| row.name=="Root").unwrap().declaration_id;
    let runtime = runtime(owner).with_durability(workflow::DurabilitySelection::Ephemeral).unwrap();
    let package = runtime.modeling_package(rows, physical).await.unwrap();
    let values = [1.0,2.0,3.0,4.0];
    let points = values.iter().enumerate().map(|(index,value)| workflow::StudyPoint {
        operation:workflow::OperationRequest::DeclaredCase(workflow::CaseOperation {
            case:root, route:pse_model::generated::enums::ModelingAnalysisRoute::Steady,
            settings:pse_runtime::math::settings::SolveSettings {
                backend:Some(Backend::Kinsol), intent:SolveIntent::Root,
                presolve:native::presolve::PolicyKind::Off,
                controls:native::solve::Controls {threads:1,..Default::default()},
                ..Default::default()
            },
        }),
        preparation:workflow::PreparationSettings {compiler:compiler(),..Default::default()},
        overlay:workflow::PointOverlay {assignments:vec![workflow::BindingAssignment {
            target:workflow::BindingTarget::Path("t".into()),
            value:workflow::BindingQuantity {magnitude:FiniteBound::try_new(*value).unwrap(),quantity:scalar.as_id(),unit},
        }]},
        policy:workflow::StudyPointPolicy {
            key:OccurrenceKey(index as u32), attempt_limit:1,
            dependencies:if continuation && index>0 {vec![Dependency::Ordering(OccurrenceKey(index as u32-1))]} else {vec![]},
            start:if continuation && index>0 {StartPolicy::Continuation(SeedEdge {
                predecessor:OccurrenceKey(index as u32-1),role:SeedRole::PrimalSolution,
                permission:ContinuationPermission::RequireUsable,unavailable:UnavailableSeedPolicy::Refuse,
            })} else {StartPolicy::Fresh},
        },
    }).collect::<Vec<_>>();
    let demand = if compact {workflow::StudyOutputDemand::Compact {
        detail:workflow::StudyDetailDemand {
            relations:vec!["runtime.solve_variables".into(),"runtime.modeling_checks".into(),"runtime.solve_metrics".into(),"runtime.solve_strategy_events".into()],
            maximum_rows:1024,maximum_bytes:1<<20,..Default::default()
        },failed_detail:Default::default(),
    }} else {workflow::StudyOutputDemand::Full {}};
    let cancel = audit::cancel_source();
    let definition = package.admit_study(&workflow::StudyInputs::Explicit(points),&demand,&cancel).await.unwrap();
    assert_eq!(definition.points.len(),4);
    assert_eq!(definition.points.iter().map(|point|point.binding_hash).collect::<BTreeSet<_>>().len(),4);
    let trim = spec["trim_cache"].as_bool().unwrap_or(false);
    if trim {owner.runtime.math().clear_program_cache();}
    let execution = audit::stage("compact_study_joined_execution");
    // The public future owns cancellation and joins its issued lanes before return.
    // There is no separate externally cancellable study handle in this route.
    let report = Box::pin(package.study(&definition,4,&cancel)).await.unwrap();
    drop(execution);
    audit::checkpoint();
    assert_eq!(report.outcomes.len(),4);
    assert_eq!(report.results.len(),4);
    if trim {owner.runtime.math().clear_program_cache();}
    let mut identities = BTreeSet::new();
    for (index,expected) in values.iter().copied().enumerate() {
        let outcome = &report.outcomes[index];
        assert_eq!(outcome.key,OccurrenceKey(index as u32));
        assert_eq!(outcome.lifecycle,StudyPointState::Completed,"{outcome:?}");
        assert!(outcome.scientific.usable && outcome.diagnostic.is_none(),"{outcome:?}");
        assert_eq!(outcome.attempts.len(),1);
        if continuation && index>0 {
            assert!(matches!(outcome.start,Some(StartProvenance::Continuation {predecessor,..}) if predecessor==OccurrenceKey(index as u32-1)),"{outcome:?}");
        } else {assert!(matches!(outcome.start,Some(StartProvenance::Fresh)),"{outcome:?}");}
        let result = report.results[index].as_ref().unwrap();
        assert!(identities.insert(result.run_id()));
        assert!(result.completion().is_ok());
        let (variables,checks) = match result {
            workflow::StudyOccurrenceResult::Ephemeral(result) => {
                assert!(!compact);
                authored_success(result);
                observations.run(result,owner.runtime.math());
                (solve_variables::Row::rows(&result.table("runtime.solve_variables").unwrap()).unwrap(),
                 modeling_checks::Row::rows(&result.table("runtime.modeling_checks").unwrap()).unwrap())
            },
            workflow::StudyOccurrenceResult::Compact(result) => {
                assert!(compact);
                let mut variables=Vec::new();
                let mut cursor=result.table_cursor("runtime.solve_variables").unwrap();
                while let Some(batch)=cursor.next_batch().await.unwrap() {variables.extend(solve_variables::Row::rows(&batch).unwrap());}
                let mut checks=Vec::new();
                let mut cursor=result.table_cursor("runtime.modeling_checks").unwrap();
                while let Some(batch)=cursor.next_batch().await.unwrap() {checks.extend(modeling_checks::Row::rows(&batch).unwrap());}
                let mut cursor=result.table_cursor("runtime.solve_metrics").unwrap();
                while let Some(batch)=cursor.next_batch().await.unwrap() {observations.persisted_metrics(&solve_metrics::Row::rows(&batch).unwrap());}
                let mut cursor=result.table_cursor("runtime.solve_strategy_events").unwrap();
                while let Some(batch)=cursor.next_batch().await.unwrap() {observations.rows(&solve_strategy_events::Row::rows(&batch).unwrap());}
                (variables,checks)
            },
            workflow::StudyOccurrenceResult::Retained {..} => panic!("ephemeral compact study returned canonical result"),
        };
        let parameters=variables.iter().filter(|row|row.parameter).collect::<Vec<_>>();
        let unknowns=variables.iter().filter(|row|!row.parameter && !row.fixed).collect::<Vec<_>>();
        assert_eq!(parameters.len(),1);
        assert_eq!(unknowns.len(),1);
        near(parameters[0].value.unwrap(),expected,1e-12);
        near(unknowns[0].value.unwrap(),2.0+expected,1e-8);
        assert!(!checks.is_empty());
        assert!(checks.iter().all(|check|check.satisfied));
    }
}

pub(super) async fn run(owner: &WorkflowRuntime, spec: &Value, observations: &mut observations::Observations) {
    match spec["operation"].as_str().unwrap() {
        "compact-root" => roots(owner, spec, observations).await,
        "compact-lp" | "compact-milp" | "compact-minlp" | "compact-uno" => classes(owner, spec["operation"].as_str().unwrap(), observations).await,
        "compact-dynamics" => trajectory(owner, spec, observations).await,
        "compact-decay" | "compact-simultaneous" => decay(owner, spec, observations).await,
        "compact-reference" | "compact-property-value" => reference(owner, spec["operation"].as_str().unwrap(), observations).await,
        "compact-implicit-derivatives" => implicit_derivatives(spec),
        "compact-implicit" => implicit_root(owner, observations).await,
        "compact-study" => study(owner, spec, observations).await,
        operation => panic!("unknown compact operation {operation}"),
    }
}
