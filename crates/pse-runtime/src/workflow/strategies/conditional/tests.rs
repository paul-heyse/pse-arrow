// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Focused admission and one-unit execution controls; no recycle iteration journey.
use super::*;
use crate::workflow::tests::{compiler_profile, id, physical, runtime};
use pse_structural::flowsheet::{Decision, Policy};

#[tokio::test]
async fn explicit_map_admission_requires_free_branch_controls_without_promoting_value() {
    for (index, declaration, allowed) in [
        (
            0,
            "var switching:Scalar; eq switch_value:switching==1; annotation start switching(1);",
            false,
        ),
        (1, "param switching:Scalar=1;", true),
    ] {
        let runtime = runtime();
        let source = "package p {def Root {CONTROL var x:Scalar; let result:Scalar=(if switching>0 then 1 else -1); state incoming supplied(true) {coordinate value=x; transport value=x tolerance 1e-7{1};} state outgoing supplied(false) {coordinate value=result; transport value=result tolerance 1e-7{1};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(1);}}".replace("CONTROL", declaration);
        let declarations = pse_authoring::language::parse(
            &source,
            id(91 + index),
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(declarations, physical()).unwrap();
        let cancel = crate::CancelSource::new();
        let analysis = package
            .declared_execution(
                root,
                compiler_profile(),
                super::super::tests::profile(SolveIntent::Root),
                Default::default(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap()
            .analysis;
        let prepared = package
            .prepare(
                root,
                analysis.instance,
                analysis.bindings.clone(),
                analysis.limits,
                &cancel,
            )
            .await
            .unwrap();
        let model = &prepared.compiled().model;
        let port = |name: &str| {
            model
                .material_ports
                .values()
                .find(|port| port.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        let selection = ModelingFlowSelection {
            nodes: BTreeSet::from([analysis.instance]),
            connections: model
                .connections
                .keys()
                .map(|&id| {
                    (
                        id,
                        Decision {
                            id,
                            cost: 1.0,
                            policy: Policy::Mandatory,
                        },
                    )
                })
                .collect(),
        };
        let request = RecycleRequest {
            tears: selection.connections.keys().copied().collect(),
            units: vec![CausalUnitRequest {
                node: analysis.instance.as_id(),
                inputs: BTreeSet::from([port("inlet")]),
                outputs: BTreeSet::from([port("outlet")]),
                realization: CausalUnitRealization::ExplicitMap,
            }],
            anderson: 0,
            damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
        };
        let result = package
            .prepare_recycle(&analysis, selection, request, &cancel)
            .await;
        if allowed {
            let admitted = result.unwrap();
            let assembly = &admitted.programs[0].program.assembly;
            assert_eq!(assembly.order(), DerivativeOrder::Value);
            assert!(
                assembly
                    .supports()
                    .iter()
                    .all(|support| support.support().first.is_empty())
            );
        } else {
            let error = result.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("causal function depends on an undeclared free input"),
                "{error}"
            );
        }
    }
}

#[tokio::test]
async fn conditional_unit_solves_original_rows_and_restores_each_boundary_overlay() {
    let runtime = runtime();
    let declarations = pse_authoring::language::parse(
        "package p {def Root {var x:Scalar; var z:Scalar; var y:Scalar; var w:Scalar; eq local:y==x/2+1; eq aux:w==z+3; let result:Scalar=y*2; state incoming supplied(true) {coordinate value=x; coordinate auxiliary=z; transport value=x tolerance 1e-7{1}; transport auxiliary=z tolerance 1e-7{1};} state outgoing supplied(false) {coordinate value=result; coordinate auxiliary=w; transport value=result tolerance 1e-7{1}; transport auxiliary=w tolerance 1e-7{1};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(4); annotation start z(2); annotation start y(1); annotation start w(1);}}",
        id(85), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = declarations
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime.modeling_package(declarations, physical()).unwrap();
    let cancel = crate::CancelSource::new();
    let analysis = package
        .declared_execution(
            root,
            compiler_profile(),
            super::super::tests::profile(SolveIntent::Root),
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    let model = package
        .prepare(
            root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let model = &model.compiled().model;
    let port = |name: &str| {
        model
            .material_ports
            .values()
            .find(|p| p.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
    };
    let symbol = |name: &str| {
        model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let row = |name: &str| {
        model
            .equations
            .iter()
            .find(|r| r.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let unknowns = BTreeSet::from([symbol("y"), symbol("w")]);
    let rows = BTreeSet::from([row("local"), row("aux")]);
    let coordinate = |port: &pse_modeling::specialize::MaterialPort, name: &str| {
        *port
            .coordinates
            .iter()
            .find(|(key, _)| key.name == name)
            .unwrap()
            .1
    };
    let inlet = coordinate(port("inlet"), "value");
    let auxiliary = coordinate(port("inlet"), "auxiliary");
    let outlet = coordinate(port("outlet"), "value");
    let auxiliary_outlet = coordinate(port("outlet"), "auxiliary");
    assert!(expand_unit_ports(model, &BTreeSet::from([port("inlet").id, inlet])).is_err());
    let node = analysis.instance.as_id();
    let selection = ModelingFlowSelection {
        nodes: BTreeSet::from([analysis.instance]),
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.0,
                        policy: Policy::Mandatory,
                    },
                )
            })
            .collect(),
    };
    let solver = crate::math::settings::SolveSettings {
        intent: SolveIntent::Root,
        ..Default::default()
    };
    let request = RecycleRequest {
        tears: selection.connections.keys().copied().collect(),
        units: vec![CausalUnitRequest {
            node,
            inputs: BTreeSet::from([port("inlet").id]),
            outputs: BTreeSet::from([port("outlet").id]),
            realization: CausalUnitRealization::Conditional {
                residuals: rows.clone(),
                unknowns: unknowns.clone(),
                solver: Box::new(solver),
            },
        }],
        anderson: 0,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let mut overlapping = request.clone();
    overlapping.units[0].inputs.insert(inlet);
    let refusal = package
        .prepare_recycle(&analysis, selection.clone(), overlapping, &cancel)
        .await
        .unwrap_err();
    let WorkflowError::ConditionalAdmission { diagnostic, .. } = refusal else {
        panic!("conditional aggregate admission lost its typed diagnostic")
    };
    assert_eq!(
        diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionInvalidModel
    );
    assert!(diagnostic.sources.contains(&port("inlet").id));
    assert!(diagnostic.sources.contains(&inlet));
    assert!(unknowns.iter().all(|id| diagnostic.sources.contains(id)));
    assert!(rows.iter().all(|id| diagnostic.sources.contains(id)));
    let prepared = package
        .prepare_recycle(&analysis, selection.clone(), request.clone(), &cancel)
        .await
        .unwrap();
    let program = prepared.programs[0].clone();
    let original = program.values.identity();
    assert_eq!(prepared.request().units[0].inputs, request.units[0].inputs);
    assert_eq!(prepared.resolved_units().next().unwrap().inputs.len(), 2);
    assert_eq!(prepared.graph.declaration().connections.len(), 1);
    assert_eq!(
        prepared.graph.bindings()[&prepared.graph.declaration().connections[0].id].len(),
        2
    );
    assert_eq!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .boundary
            .members
            .rows,
        rows.into_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .boundary
            .members
            .columns,
        unknowns.into_iter().collect::<Vec<_>>()
    );
    assert_eq!(program.program.assembly.order(), DerivativeOrder::Value);
    let controls = Controls::default();
    let execution = Execution::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        &controls,
    );
    let budget = crate::math::WorkerBudget::new(16 << 20);
    let worker = runtime
        .native()
        .worker(
            program.program.clone(),
            &prepared.providers,
            execution.scope().unwrap(),
            &budget,
        )
        .unwrap();
    let mut unit = UnitWorker {
        input_ids: program.inputs.iter().map(|i| i.0).collect(),
        output_ids: program.outputs.iter().map(|o| o.0).collect(),
        program,
        worker,
        service: runtime.native().clone(),
        providers: prepared.providers,
        budget,
    };
    let charged = unit.budget.used();
    for (input, expected) in [(4.0, 6.0), (8.0, 10.0), (4.0, 6.0)] {
        let outputs = unit
            .evaluate(
                &BTreeMap::from([(inlet, input), (auxiliary, 2.0)]),
                &execution,
            )
            .unwrap();
        assert!((outputs[&outlet] - expected).abs() < 1e-7);
        assert!((outputs[&auxiliary_outlet] - 5.0).abs() < 1e-7);
        assert_eq!(unit.program.values.identity(), original);
        assert_eq!(unit.budget.used(), charged);
    }
    assert!(unit.evaluate(&BTreeMap::new(), &execution).is_err());
    assert!(
        unit.evaluate(
            &BTreeMap::from([(inlet, f64::NAN), (auxiliary, 2.0)]),
            &execution
        )
        .is_err()
    );
    execution
        .cancel
        .store(true, std::sync::atomic::Ordering::Release);
    assert!(
        unit.evaluate(&BTreeMap::from([(inlet, 4.0)]), &execution)
            .is_err()
    );
    assert_eq!(unit.program.values.identity(), original);
    assert_eq!(unit.budget.used(), charged);

    let mut explicit = request.clone();
    explicit.units[0].realization = CausalUnitRealization::ExplicitMap;
    assert!(
        package
            .prepare_recycle(&analysis, selection.clone(), explicit, &cancel)
            .await
            .is_err()
    );
    let mut invalid = request;
    if let CausalUnitRealization::Conditional { solver, .. } = &mut invalid.units[0].realization {
        solver.backend = Some(Backend::Highs);
    }
    assert!(
        package
            .prepare_recycle(&analysis, selection, invalid, &cancel)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn conditional_unit_derived_boundary_solves_constituent_variables() {
    let runtime = runtime();
    let declarations = pse_authoring::language::parse(
        "package p {def Root {var x:Scalar; var z:Scalar; var y:Scalar; var w:Scalar; eq local:y==x/2+1; eq aux:w==z+3; let result:Scalar=y*2; let boundary:Scalar=x*2; state incoming supplied(true) {coordinate value=boundary; coordinate auxiliary=z; transport value=boundary tolerance 1e-7{1}; transport auxiliary=z tolerance 1e-7{1};} state outgoing supplied(false) {coordinate value=result; coordinate auxiliary=w; transport value=result tolerance 1e-7{1}; transport auxiliary=w tolerance 1e-7{1};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(4); annotation start z(2); annotation start y(1); annotation start w(1);}}",
        id(85), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = declarations
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime.modeling_package(declarations, physical()).unwrap();
    let cancel = crate::CancelSource::new();
    let analysis = package
        .declared_execution(
            root,
            compiler_profile(),
            super::super::tests::profile(SolveIntent::Root),
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    let model = package
        .prepare(
            root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let model = &model.compiled().model;
    let port = |name: &str| {
        model
            .material_ports
            .values()
            .find(|p| p.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
    };
    let symbol = |name: &str| {
        model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let row = |name: &str| {
        model
            .equations
            .iter()
            .find(|r| r.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let unknowns = BTreeSet::from([symbol("x"), symbol("y"), symbol("w")]);
    let rows = BTreeSet::from([row("local"), row("aux")]);
    let coordinate = |port: &pse_modeling::specialize::MaterialPort, name: &str| {
        *port
            .coordinates
            .iter()
            .find(|(key, _)| key.name == name)
            .unwrap()
            .1
    };
    let inlet = coordinate(port("inlet"), "value");
    let auxiliary = coordinate(port("inlet"), "auxiliary");
    let outlet = coordinate(port("outlet"), "value");
    let auxiliary_outlet = coordinate(port("outlet"), "auxiliary");
    assert!(expand_unit_ports(model, &BTreeSet::from([port("inlet").id, inlet])).is_err());
    let node = analysis.instance.as_id();
    let selection = ModelingFlowSelection {
        nodes: BTreeSet::from([analysis.instance]),
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.0,
                        policy: Policy::Mandatory,
                    },
                )
            })
            .collect(),
    };
    let solver = crate::math::settings::SolveSettings {
        intent: SolveIntent::Root,
        ..Default::default()
    };
    let request = RecycleRequest {
        tears: selection.connections.keys().copied().collect(),
        units: vec![CausalUnitRequest {
            node,
            inputs: BTreeSet::from([port("inlet").id]),
            outputs: BTreeSet::from([port("outlet").id]),
            realization: CausalUnitRealization::Conditional {
                residuals: rows.clone(),
                unknowns: unknowns.clone(),
                solver: Box::new(solver),
            },
        }],
        anderson: 0,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let prepared = package
        .prepare_recycle(&analysis, selection.clone(), request.clone(), &cancel)
        .await
        .unwrap();
    let program = prepared.programs[0].clone();
    let original = program.values.identity();
    assert!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .plan
            .structure()
            .parameters()
            .iter()
            .any(|p| p.id == pse_ids::named_id(symbol("boundary"), "conditional-boundary-value"))
    );
    assert_eq!(prepared.request().units[0].inputs, request.units[0].inputs);
    assert_eq!(prepared.resolved_units().next().unwrap().inputs.len(), 2);
    assert_eq!(prepared.graph.declaration().connections.len(), 1);
    assert_eq!(
        prepared.graph.bindings()[&prepared.graph.declaration().connections[0].id].len(),
        2
    );
    assert_eq!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .boundary
            .members
            .rows,
        rows.into_iter()
            .chain(std::iter::once(
                ModelingOutput::ConditionalBoundary(symbol("boundary")).row_id()
            ))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .boundary
            .members
            .columns,
        unknowns.into_iter().collect::<Vec<_>>()
    );
    assert_eq!(program.program.assembly.order(), DerivativeOrder::Value);
    let controls = Controls::default();
    let execution = Execution::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        &controls,
    );
    let budget = crate::math::WorkerBudget::new(16 << 20);
    let worker = runtime
        .native()
        .worker(
            program.program.clone(),
            &prepared.providers,
            execution.scope().unwrap(),
            &budget,
        )
        .unwrap();
    let mut unit = UnitWorker {
        input_ids: program.inputs.iter().map(|i| i.0).collect(),
        output_ids: program.outputs.iter().map(|o| o.0).collect(),
        program,
        worker,
        service: runtime.native().clone(),
        providers: prepared.providers,
        budget,
    };
    let charged = unit.budget.used();
    for (input, expected) in [(4.0, 4.0), (8.0, 6.0), (4.0, 4.0)] {
        let outputs = unit
            .evaluate(
                &BTreeMap::from([(inlet, input), (auxiliary, 2.0)]),
                &execution,
            )
            .unwrap();
        assert!((outputs[&outlet] - expected).abs() < 1e-7);
        assert!((outputs[&auxiliary_outlet] - 5.0).abs() < 1e-7);
        assert_eq!(unit.program.values.identity(), original);
        assert_eq!(unit.budget.used(), charged);
    }
    assert!(unit.evaluate(&BTreeMap::new(), &execution).is_err());
    assert!(
        unit.evaluate(
            &BTreeMap::from([(inlet, f64::NAN), (auxiliary, 2.0)]),
            &execution
        )
        .is_err()
    );
    execution
        .cancel
        .store(true, std::sync::atomic::Ordering::Release);
    assert!(
        unit.evaluate(&BTreeMap::from([(inlet, 4.0)]), &execution)
            .is_err()
    );
    assert_eq!(unit.program.values.identity(), original);
    assert_eq!(unit.budget.used(), charged);

    let mut explicit = request.clone();
    explicit.units[0].realization = CausalUnitRealization::ExplicitMap;
    assert!(
        package
            .prepare_recycle(&analysis, selection.clone(), explicit, &cancel)
            .await
            .is_err()
    );
    let mut invalid = request;
    if let CausalUnitRealization::Conditional { solver, .. } = &mut invalid.units[0].realization {
        solver.backend = Some(Backend::Highs);
    }
    assert!(
        package
            .prepare_recycle(&analysis, selection, invalid, &cancel)
            .await
            .is_err()
    );
}

#[test]
fn conditional_unit_request_has_one_declared_realization_and_canonical_settings_defaults() {
    let request = CausalUnitRequest {
        node: id(1),
        inputs: BTreeSet::from([id(2)]),
        outputs: BTreeSet::from([id(3)]),
        realization: CausalUnitRealization::Conditional {
            residuals: BTreeSet::from([id(4)]),
            unknowns: BTreeSet::from([id(5)]),
            solver: Box::new(crate::math::settings::SolveSettings {
                intent: SolveIntent::Root,
                ..Default::default()
            }),
        },
    };
    let value = serde_json::to_value(&request).unwrap();
    let decoded: CausalUnitRequest = serde_json::from_value(value.clone()).unwrap();
    let CausalUnitRealization::Conditional { solver, .. } = decoded.realization else {
        panic!("changed realization")
    };
    let full = solver.profile().unwrap();
    let omitted: crate::math::settings::SolveSettings =
        serde_json::from_value(serde_json::json!({"version":1,"intent":"root"})).unwrap();
    assert_eq!(
        crate::math::solves::profile_key(&full).unwrap(),
        crate::math::solves::profile_key(&omitted.profile().unwrap()).unwrap()
    );
    let mut conflict = value;
    conflict["realization"]["fallback"] = serde_json::json!("simultaneous");
    assert!(serde_json::from_value::<CausalUnitRequest>(conflict).is_err());
}

#[test]
fn conditional_unit_admission_diagnostic_retains_sources_and_original_cause() {
    use std::error::Error;
    let unit = CausalUnitRequest {
        node: id(1),
        inputs: BTreeSet::from([id(2)]),
        outputs: BTreeSet::from([id(3)]),
        realization: CausalUnitRealization::Conditional {
            residuals: BTreeSet::from([id(4)]),
            unknowns: BTreeSet::from([id(5)]),
            solver: Default::default(),
        },
    };
    let request = RecycleRequest {
        tears: BTreeSet::new(),
        units: vec![unit.clone()],
        anderson: 0,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let cause = MathRuntimeError::Compile(pse_compiler::workspace::CompileError::Missing(
        "external row couples selected unknown".into(),
    ));
    let error = conditional_admission(&unit, &request, cause);
    let diagnostic = error.boundary_diagnostic();
    assert_eq!(
        diagnostic.class,
        pse_model::diagnostic::BoundaryClass::InvalidModel
    );
    assert_eq!(
        diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionInvalidModel
    );
    assert_eq!(diagnostic.sources, vec![id(1), id(2), id(3), id(4), id(5)]);
    assert!(
        error
            .source()
            .unwrap()
            .to_string()
            .contains("external row couples")
    );
    let encoded = serde_json::to_vec(&diagnostic).unwrap();
    let decoded: pse_model::diagnostic::BoundaryDiagnostic =
        serde_json::from_slice(&encoded).unwrap();
    let numerical = conditional_admission(
        &unit,
        &request,
        native::ProblemError::numerical("native iteration stagnated").into(),
    );
    let numerical_diagnostic = numerical.boundary_diagnostic();
    assert_eq!(
        numerical_diagnostic.class,
        pse_model::diagnostic::BoundaryClass::Numerical
    );
    assert_eq!(
        numerical_diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionNumerical
    );
    assert!(
        numerical
            .source()
            .unwrap()
            .to_string()
            .contains("stagnated")
    );
    let nonfinite = conditional_admission(
        &unit,
        &request,
        pse_math::MathError::Evaluation {
            source_id: id(6),
            order: DerivativeOrder::Value,
            detail: "original boundary observation was nonfinite".into(),
        }
        .into(),
    );
    let observed = nonfinite.boundary_diagnostic();
    // The expression owner classifies this actual evaluation failure as a rejected
    // trial. Preserve that classification; an admission wrapper must not relabel it.
    assert_eq!(
        observed.class,
        pse_model::diagnostic::BoundaryClass::TrialRejected
    );
    assert_eq!(
        observed.rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionTrialRejected
    );
    assert!(observed.sources.contains(&id(6)));
    assert!(
        nonfinite
            .source()
            .unwrap()
            .to_string()
            .contains("nonfinite")
    );
    assert_eq!(decoded.sources, diagnostic.sources);
}

#[tokio::test]
async fn conditional_unit_affine_boundary_retains_difference_magnitudes_and_point_outputs() {
    use pse_model::generated::enums::{NumericalCoordinates, NumericalSource, NumericalTarget};
    let runtime = runtime();
    let declarations = pse_authoring::language::parse(
        "package p {def Root {var x:Temperature; var y:Temperature; eq local:y==x; let boundary:Temperature=x; state incoming supplied(true) {coordinate value=boundary; transport value=boundary tolerance 1e-7{K};} state outgoing supplied(false) {coordinate value=y; transport value=y tolerance 1e-7{K};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(300{K}); annotation start y(300{K});}}",
        id(86), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = declarations
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime.modeling_package(declarations, physical()).unwrap();
    let cancel = crate::CancelSource::new();
    let mut analysis = package
        .declared_execution(
            root,
            compiler_profile(),
            super::super::tests::profile(SolveIntent::Root),
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    let prepared_model = package
        .prepare(
            root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let model = &prepared_model.compiled().model;
    let symbol = |name: &str| {
        model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let port = |name: &str| {
        model
            .material_ports
            .values()
            .find(|p| p.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
    };
    let incoming = port("inlet");
    let outgoing = port("outlet");
    let inlet = *incoming.coordinates.values().next().unwrap();
    let outlet = *outgoing.coordinates.values().next().unwrap();
    let boundary = symbol("boundary");
    let point = pse_quantity::standard::ids::quantity("temperature.point");
    let difference = pse_quantity::standard::ids::quantity("temperature.difference");
    analysis
        .numerical
        .targets
        .push(pse_math::numerics::TargetSpec {
            id: boundary,
            kind: NumericalTarget::Observable,
            quantity: point,
            unit: pse_quantity::standard::ids::unit("K"),
            integer: false,
            declared_tolerance: None,
        });
    analysis
        .numerical
        .declarations
        .push(pse_math::numerics::SourcedRequirement {
            source: NumericalSource::Model,
            declaration: pse_model::numerics::NumericalRequirement {
                requirement_id: id(87),
                model_id: Some(root.as_id().into()),
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: boundary,
                target_kind: NumericalTarget::Observable,
                nominal: Some(8.0),
                scaling_factor: Some(0.125),
                absolute_tolerance: Some(0.08),
                relative_tolerance: Some(0.01),
                unit_id: Some(pse_quantity::standard::ids::unit("degF").as_id()),
                coordinates: NumericalCoordinates::Physical,
                priority: 0,
                required: true,
                provenance: "explicit point-observation magnitudes in Fahrenheit".into(),
            },
        });
    let selection = ModelingFlowSelection {
        nodes: BTreeSet::from([analysis.instance]),
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.0,
                        policy: Policy::Mandatory,
                    },
                )
            })
            .collect(),
    };
    let request = RecycleRequest {
        tears: selection.connections.keys().copied().collect(),
        units: vec![CausalUnitRequest {
            node: analysis.instance.as_id(),
            inputs: BTreeSet::from([incoming.id]),
            outputs: BTreeSet::from([outgoing.id]),
            realization: CausalUnitRealization::Conditional {
                residuals: model
                    .equations
                    .iter()
                    .filter(|r| r.lineage.path.ends_with(".local"))
                    .map(|r| r.id)
                    .collect(),
                unknowns: BTreeSet::from([symbol("x"), symbol("y")]),
                solver: Box::new(crate::math::settings::SolveSettings {
                    intent: SolveIntent::Root,
                    ..Default::default()
                }),
            },
        }],
        anderson: 0,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let prepared = package
        .prepare_recycle(&analysis, selection, request, &cancel)
        .await
        .unwrap();
    let program = prepared.programs[0].clone();
    let conditional = program.conditional.as_ref().unwrap();
    let boundary_row = ModelingOutput::ConditionalBoundary(boundary).row_id();
    let row = conditional
        .view
        .plan
        .structure()
        .rows()
        .iter()
        .find(|r| r.id == boundary_row)
        .unwrap();
    assert_eq!(row.quantity, difference);
    assert_eq!((row.lower, row.upper), (0.0, 0.0));
    assert_eq!(
        conditional
            .view
            .plan
            .structure()
            .parameters()
            .iter()
            .find(|p| p.id == pse_ids::named_id(boundary, "conditional-boundary-value"))
            .unwrap()
            .quantity,
        point
    );
    let (scale, tolerance) = conditional.row_magnitudes(boundary_row).unwrap();
    assert!((scale - 8.0 * 5.0 / 9.0).abs() < 1e-12);
    assert!((tolerance - (0.08 + 0.01 * 8.0) * 5.0 / 9.0).abs() < 1e-12);
    assert_eq!(
        program
            .program
            .assembly
            .structure()
            .rows()
            .iter()
            .find(|r| r.id == ModelingOutput::Member(symbol("y")).row_id())
            .unwrap()
            .quantity,
        point
    );
    let original = program.values.identity();
    let execution = Execution::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        &Controls::default(),
    );
    let budget = crate::math::WorkerBudget::new(16 << 20);
    let worker = runtime
        .native()
        .worker(
            program.program.clone(),
            &prepared.providers,
            execution.scope().unwrap(),
            &budget,
        )
        .unwrap();
    let mut unit = UnitWorker {
        input_ids: program.inputs.iter().map(|i| i.0).collect(),
        output_ids: program.outputs.iter().map(|o| o.0).collect(),
        program,
        worker,
        service: runtime.native().clone(),
        providers: prepared.providers,
        budget,
    };
    let charged = unit.budget.used();
    for target in [310.0, 320.0, 310.0] {
        let outputs = unit
            .evaluate(&BTreeMap::from([(inlet, target)]), &execution)
            .unwrap();
        assert!((outputs[&outlet] - target).abs() < 1e-7);
        assert_eq!(unit.program.values.identity(), original);
        assert_eq!(unit.budget.used(), charged);
    }
    assert!(
        unit.evaluate(&BTreeMap::from([(inlet, f64::NAN)]), &execution)
            .is_err()
    );
    assert!(unit.evaluate(&BTreeMap::new(), &execution).is_err());
    execution
        .cancel
        .store(true, std::sync::atomic::Ordering::Release);
    assert!(
        unit.evaluate(&BTreeMap::from([(inlet, 310.0)]), &execution)
            .is_err()
    );
    assert_eq!(unit.program.values.identity(), original);
    assert_eq!(unit.budget.used(), charged);
}
