// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::math::solves::Outcome;
#[cfg(feature = "solver-kinsol")]
use crate::workflow::tests::compiler_profile;
use crate::workflow::tests::{id, physical, runtime};
use pse_backend_native::execution::BackendSettings;
pub(super) fn profile(intent: SolveIntent) -> SolverProfile {
    SolverProfile {
        intent,
        selection: SolverSelection::Auto,
        controls: Controls::default(),
        backend: BackendSettings::Default,
        presolve: Default::default(),
        numerics: Default::default(),
        convexity: Default::default(),
        sensitivity: None,
    }
}
#[tokio::test]
async fn explicit_cone_request_runs_without_an_algebraic_compiler_flag() {
    let physical = physical();
    let q = physical.quantities.neutral_dimensionless().unwrap();
    let unit = physical.quantities.quantity_type(q).unwrap().canonical_unit;
    let port = |symbol_id| AnalysisPort {
        symbol_id,
        quantity_id: q.as_id(),
        unit_id: unit.as_id(),
    };
    let request = ConicRequest {
        variables: vec![port(id(1))],
        rows: vec![port(id(2))],
        objective_port: port(SemanticId::NIL),
        quadratic: native::conic::SparseMatrix::zeros(1, 1),
        objective: vec![1.0],
        constraints: native::conic::SparseMatrix::new(1, 1, vec![0, 1], vec![0], vec![-1.0]),
        rhs: vec![-2.0],
        cones: vec![native::conic::Cone::Nonnegative { dimension: 1 }],
        objective_constant: 3.0,
        gram_factors: vec![],
        gram_weights: vec![],
    };
    let prepared = runtime()
        .prepare_conic(request, &physical, profile(SolveIntent::Optimize))
        .await
        .unwrap();
    assert_eq!(
        prepared.solve().route(),
        native::routing::Route::Native(Backend::Clarabel)
    );
    let result = prepared.start().unwrap().finish().await.unwrap();
    let Outcome::Native(report) = &result.outcome else {
        panic!("{:?}", result.outcome)
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] - 2.0).abs() < 1e-6);
    assert!((report.candidate.as_ref().unwrap().objective.unwrap() - 5.0).abs() < 1e-6);
    assert!(report.quality.as_ref().unwrap().feasible());
    assert_eq!(
        report.qualification,
        Qualification::OptimalWithinTolerance,
        "{report:?}"
    );
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn failed_continuation_preserves_original_bindings_and_prior_solved_unknowns() {
    use std::collections::BTreeMap;
    let runtime = runtime();
    let physical = physical();
    let neutral = physical.quantities.neutral_dimensionless().unwrap();
    let rows=pse_authoring::language::parse("package p {def Root {param a:Scalar=4; var x:Scalar; var y:Scalar; eq first:x*x==a; eq second:y==x+1; annotation start x(2); annotation start y(3); annotation nominal x(2);}}",id(80),pse_authoring::language::IdentityPolicy::Named,Default::default()).unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, physical, BTreeMap::from([("Scalar".into(), neutral)]))
        .unwrap();
    let original = package.revision.identity();
    let cancel = crate::CancelSource::new();
    let analysis = package
        .declared_analysis(
            root,
            pse_model::generated::enums::ModelingAnalysisRoute::Steady,
            compiler_profile(),
            SolverProfile {
                controls: Controls {
                    start: StartPolicy::PreviousAccepted,
                    iterations: 50,
                    ..Default::default()
                },
                ..profile(SolveIntent::Root)
            },
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let model = package
        .prepare(
            root,
            pse_modeling::specialize::root_instance(root),
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let symbol = |name: &str| {
        model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let (x, y, a) = (symbol("x"), symbol("y"), symbol("a"));
    let prepared = package
        .prepare_block_initialization(
            &analysis,
            vec![BTreeMap::from([(a, 9.0)]), BTreeMap::from([(a, -1.0)])],
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(prepared.blocks().boundaries().count(), 2);
    let report = prepared.start().unwrap().finish().await.unwrap();
    assert_eq!(report.original.scalars[&a], 4.0);
    assert_eq!(report.original.scalars[&x], 2.0);
    assert_eq!(report.completed_stages, 1, "{report:?}");
    assert!((report.values.scalars[&x] - 3.0).abs() < 1e-6);
    assert!(!report.values.scalars.contains_key(&a));
    assert_eq!(report.stages[1].candidate.scalars[&a], -1.0);
    assert!(!report.stages[1].completed);
    assert!(!report.original_bindings_restored);
    assert_eq!(package.revision.identity(), original);
    assert!((report.values.scalars[&y] - 4.0).abs() < 1e-6);
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn authored_causal_recycle_retains_topology_and_refuses_hidden_inputs() {
    use pse_structural::flowsheet::{Decision, Policy};
    use std::collections::{BTreeMap, BTreeSet};
    let runtime = runtime();
    let physical = physical();
    let neutral = physical.quantities.neutral_dimensionless().unwrap();
    let rows=pse_authoring::language::parse("package p {def Root {param a:Scalar=2; var x:Scalar; var hidden:Scalar; let output:Scalar=x/2+a; let bad:Scalar=x+hidden; port inlet:Scalar=x; annotation connectivity inlet(1,0); port outlet:Scalar=output; annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(1); annotation nominal x(5); annotation start hidden(0);}}",id(81),pse_authoring::language::IdentityPolicy::Named,Default::default()).unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, physical, BTreeMap::from([("Scalar".into(), neutral)]))
        .unwrap();
    let cancel = crate::CancelSource::new();
    let mut analysis = package
        .declared_analysis(
            root,
            pse_model::generated::enums::ModelingAnalysisRoute::Steady,
            compiler_profile(),
            profile(SolveIntent::Root),
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    analysis.bindings.demand.push("bad".into());
    let model = package
        .prepare(
            root,
            pse_modeling::specialize::root_instance(root),
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let product = &model.compiled().model;
    let port = |name: &str| {
        product
            .ports
            .values()
            .find(|p| p.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let (input, output) = (port("inlet"), port("outlet"));
    let x = product.ports[&input].symbol;
    analysis.solver.numerics.requirements.push(serde_json::from_value(serde_json::json!({"requirement_id":id(91),"model_id":root,"case_id":root,"target_id":x,"target_kind":"variable","nominal":100.0,"scaling_factor":null,"absolute_tolerance":1e-9,"relative_tolerance":1e-10,"unit_id":null,"coordinates":"physical","priority":0,"required":true,"provenance":"explicit source-coordinate acceptance"})).unwrap());

    let selection = pse_compiler::workspace::ModelingFlowSelection {
        nodes: BTreeSet::from([pse_modeling::specialize::root_instance(root)]),
        connections: product
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 2.0,
                        policy: Policy::Mandatory,
                    },
                )
            })
            .collect(),
    };
    let flow = package
        .prepare_flow(&analysis, selection.clone(), &cancel)
        .await
        .unwrap();
    let witness = runtime
        .native()
        .select_tears(
            flow,
            crate::math::flows::TearMethod::UnweightedHeuristic,
            Controls::default(),
        )
        .unwrap()
        .finish()
        .await
        .unwrap()
        .selected
        .clone()
        .unwrap();
    assert_eq!(witness.cost, 2.0);
    let request = RecycleRequest {
        tears: witness.decisions,
        units: vec![CausalUnitRequest {
            node: root.as_id(),
            inputs: BTreeSet::from([input]),
            outputs: BTreeSet::from([output]),
        }],
        anderson: 1,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let prepared = package
        .prepare_recycle(&analysis, selection.clone(), request.clone(), &cancel)
        .await
        .unwrap();
    assert_eq!(prepared.order().unwrap(), vec![root.as_id()]);
    assert_eq!(prepared.numerics().targets.len(), 2);
    assert!(prepared.numerics().targets.iter().all(|t| {
        t.nominal == 100.0
            && (t.budget - 1.1e-8).abs() < 1e-20
            && t.provenance
                .iter()
                .any(|p| p.selected && p.declaration == Some(id(91)))
    }));

    let report = prepared.start().unwrap().finish().await.unwrap();
    assert_eq!(
        report.report.qualification,
        Qualification::Feasible,
        "{:?}",
        report.report
    );
    assert!((report.report.candidate.as_ref().unwrap().primal[0] - 4.0).abs() < 1e-6);
    assert!(
        report
            .report
            .metrics
            .contains_key("KINGetNumNonlinSolvIters")
    );
    let original = package.revision.identity();
    let mut rows = package.declarations().to_vec();
    rows.iter_mut()
        .find(|r| r.name == "output")
        .unwrap()
        .value
        .binding
        .as_mut()
        .unwrap()
        .expression = Some("x/2+a+hidden".into());
    let revised = package.with_declarations(rows).unwrap();
    assert_ne!(revised.revision.identity(), original);
    assert_eq!(package.revision.identity(), original);
    let error = revised
        .prepare_recycle(&analysis, selection, request, &cancel)
        .await
        .unwrap_err();
    assert!(
        error.to_string().contains("undeclared free input"),
        "{error}"
    );
}
