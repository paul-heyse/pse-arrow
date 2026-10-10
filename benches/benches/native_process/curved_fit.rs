// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public two-parameter IDAS fitting with exact curvature and independent curve data.
use super::*;
use pse_backend_native::{self as native, solve::HessianMode};
use pse_ids::{SemanticId, named_id};
use pse_model::generated::{enums::NumericalTarget, identities::FitId};
use pse_runtime::workflow;
use serde_json::json;

fn id(name: &str) -> SemanticId {
    named_id(SemanticId::NIL, &format!("process-cost.curved-fit.{name}"))
}

/// Independent original-coordinate solution of x'=-k*x², x(0)=a, z=k*x².
fn curve(member: &str, time: f64, k: f64, a: f64) -> f64 {
    let x = a / (1.0 + a * k * time);
    match member {
        "x" => x,
        "z" => k * x * x,
        _ => panic!("unknown curve member"),
    }
}

pub(super) async fn run(
    owner: &WorkflowRuntime,
    observations: &mut observations::Observations,
    gradient_only: bool,
) {
    let physical = physical(owner).await;
    let observed = [
        ("reading-0", "x", 0.5),
        ("reading-1", "x", 1.0),
        ("reading-2", "x", 1.5),
        ("reading-3", "x", 2.0),
        ("reading-4", "z", 1.0),
    ];
    let sigma = 0.1;
    let mut source = String::from(
        "package curved_fit {
        entity kind origin provenance {attribute title:Text;}
        entity origin analytic {title=\"closed-form x=a/(1+a*k*t), z=k*x*x\"}
        enum role {measured facets(measured)}
        entity kind reading {attribute value:Scalar?; attribute sigma:Scalar?;}
        test Decay fixture {
          dof 0; route integrated; procedure integrate;
          integrate samples(0{s},2{s}) relative(global)
            normalized_absolute(global) step(1e-5{s});
        } {
          domain t:Time from 0{s} to 2{s};
          discretize grid on t using integrated(elements=1,order=1);
          param k:Scalar=1; param a:Scalar=2;
          var x[i in t]:Scalar; var z[i in t]:Scalar;
          eq rate[i in t]:d(x[i])/di==-z[i]/1{s};
          eq closure[i in t]:z[i]==k*x[i]*x[i];
          eq initial:x[0{s}]==a;
          annotation start x(1); annotation start z(1);
          annotation report x(\"state\"); annotation report z(\"closure\");
        } ",
    );
    for (index, (name, member, time)) in observed.iter().enumerate() {
        let value = curve(member, *time, 1.3, 1.8);
        source.push_str(&format!(
            "@id(\"{}\") entity reading sample{index} provenance(analytic,role.measured) {{value={value},sigma={sigma}}} ",
            id(name)
        ));
    }
    source.push('}');
    let rows = pse_authoring::language::parse(
        &source,
        id("package"),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let case = rows
        .iter()
        .find(|row| row.name == "Decay")
        .unwrap()
        .declaration_id;
    let mut declarations = workflow::FitDeclarations::default();
    declarations.fits.push(serde_json::from_value(json!({
        "fit_id":id("fit"),
        "parameters":[
            {"symbol_id":id("k"),"fixed":false,"value":1.0,"lower":0.1,"upper":10.0,"scale":1.0},
            {"symbol_id":id("a"),"fixed":false,"value":2.0,"lower":0.1,"upper":10.0,"scale":1.0}
        ],
        "experiments":[{"experiment_id":id("experiment"),"case_id":case,"route":"integrated","bindings":[
            {"parameter_id":id("k"),"path":"k"},
            {"parameter_id":id("a"),"path":"a"}
        ]}],
        "observations":observed.iter().map(|(name,member,time)|json!({
            "observation_id":id(name),"experiment_id":id("experiment"),
            "value_attribute":"value","standard_deviation_attribute":"sigma",
            "output_path":format!("{member}[0{{s}}]"),"time":time,"time_basis":"model_clock",
            "included":true,"importance":1.0
        })).collect::<Vec<_>>()
    })).unwrap());
    let package = runtime(owner)
        .modeling_package(rows, physical)
        .await
        .unwrap()
        .with_fit_declarations(declarations)
        .await
        .unwrap();
    let cancel = CancelSource::new();
    let simulation = package
        .declared_simulation(case, compiler(), None, seed_limits(), &cancel)
        .await
        .unwrap();
    assert_eq!(simulation.contract().states.len(), 2);
    assert_eq!(simulation.contract().parameters.len(), 2);
    // Both observations are exact state members. Read their actual frozen
    // physical budgets, independent of the native local integration controls.
    let budget = |differential| {
        let mut states = simulation
            .contract()
            .states
            .iter()
            .zip(&simulation.contract().differential)
            .filter(|(_, flag)| **flag == differential);
        let (&state, _) = states.next().unwrap();
        assert!(states.next().is_none());
        assert!(
            simulation
                .contract()
                .outputs
                .contains(&pse_compiler::workspace::ModelingOutput::Member(state).row_id())
        );
        let target = simulation
            .numerics()
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Variable && target.id == state)
            .unwrap();
        assert!(target.budget.is_finite() && target.budget > 0.0);
        target.budget
    };
    let output_budgets = [budget(true), budget(false)];
    let mut integration = simulation.profile().clone();
    integration.method = native::dynamics::Method::Idas;
    let mut solver = profile(Backend::Ipopt, true);
    let hessian = if gradient_only {
        HessianMode::LimitedMemory
    } else {
        HessianMode::Exact
    };
    let derivatives = if gradient_only {
        workflow::FitDerivatives::Gradient
    } else {
        workflow::FitDerivatives::Responses
    };
    solver.controls.hessian = hessian;
    solver.presolve = native::presolve::Policy::Off;
    let prepared = package
        .prepare_fit(
            FitId::from_id(id("fit")),
            workflow::FitProfile {
                solver,
                simulations: BTreeMap::from([(id("experiment").into(), integration)]),
                rank_tolerance: 1e-8,
                max_cells: 100_000,
                derivatives,
                uncertainty: None,
            },
            compiler(),
            seed_limits(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(prepared.declaration().parameters.len(), 2);
    assert!(
        prepared
            .declaration()
            .parameters
            .iter()
            .all(|parameter| !parameter.fixed)
    );
    let objective_budget = prepared
        .numerics()
        .targets
        .iter()
        .find(|target| target.kind == NumericalTarget::Objective && target.id == SemanticId::NIL)
        .unwrap()
        .budget;
    assert!(objective_budget.is_finite() && objective_budget > 0.0);
    let result = prepared.start().unwrap().wait().await.unwrap();
    observations.run(&result, owner.runtime.pool());
    let RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing curved fit")
    };
    assert!(report.estimate_qualified() && result.usable(), "{report:?}");
    assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
    assert_eq!(report.hessian, hessian);
    assert_eq!(report.derivatives, derivatives);
    let candidate = report.candidate.as_ref().unwrap();
    assert_eq!(candidate.len(), 2);
    assert_eq!(report.predictions.len(), observed.len());
    let mut independent_objective = 0.0;
    for (row, (_, member, time)) in observed.iter().enumerate() {
        let prediction = curve(member, *time, candidate[0], candidate[1]);
        let original = curve(member, *time, 1.3, 1.8);
        let allowance = output_budgets[usize::from(*member == "z")];
        near(report.predictions[row].unwrap(), prediction, allowance);
        independent_objective += 0.5 * ((prediction - original) / sigma).powi(2);
    }
    // Original-space KKT acceptance does not promise parameter recovery error.
    // Assess the independent curve at the actual candidate and its noiseless
    // optimum under the admitted prediction and objective budgets instead.
    let objective = report.objective.unwrap();
    near(objective, independent_objective, objective_budget);
    assert!(independent_objective.is_finite() && independent_objective <= objective_budget);
    assert!(
        objective.is_finite() && objective <= objective_budget,
        "{report:?}"
    );
    std::hint::black_box((&report.reports, &report.responses, &report.predictions));
}
