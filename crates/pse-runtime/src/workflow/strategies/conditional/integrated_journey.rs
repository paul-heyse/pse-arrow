// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Connected physical execution, distinct from the reference model's honest root refusal.
use super::*;
use crate::workflow::tests::{compiler_profile, physical, runtime};
use pse_structural::flowsheet::{Decision, Policy};

const SOURCE: &str = include_str!("../../../../../../tests/fixtures/indexed-recycle-closure.pse");

fn package(source: &str) -> ModelingPackage {
    let declarations = pse_authoring::language::parse(
        source,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let mut physical = physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    runtime().modeling_package(declarations, physical).unwrap()
}

#[tokio::test]
async fn indexed_mixer_holdup_separator_executes_conditional_recycle_and_external_closure() {
    let package = package(SOURCE);
    let cancel = crate::CancelSource::new();
    let root = package
        .declarations()
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
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
    let resolved = package
        .resolve_case(
            root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            analysis.case.clone(),
            DerivativeOrder::First,
            analysis.compiler,
            analysis.solver.clone(),
            analysis.numerical.clone(),
            Default::default(),
            false,
            &cancel,
        )
        .await
        .unwrap();
    let model = &resolved.model.model.compiled().model;
    let mut units = Vec::new();
    let mut nodes = BTreeSet::new();
    let mut ports = BTreeMap::new();
    for name in ["mixer", "reactor", "separator"] {
        let node = model
            .instances
            .values()
            .find(|n| {
                n.path.ends_with(&format!(".{name}")) || n.path.contains(&format!(".{name}["))
            })
            .unwrap();
        nodes.insert(node.id);
        let mut inputs = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        for port in model
            .ports
            .values()
            .filter(|p| p.lineage.instance == node.id)
        {
            let member = port.lineage.path.rsplit('.').next().unwrap();
            ports.insert(format!("{name}.{member}"), port.id);
            if member == "inlet" {
                inputs.insert(port.id);
            } else {
                outputs.insert(port.id);
            }
        }
        let symbols = inputs.iter().map(|id| model.ports[id].symbol).collect();
        let inventory =
            pse_compiler::workspace::CompilerWorkspace::modeling_conditional_unit_inventory(
                resolved.model.model.compiled(),
                resolved.model.case.compiled().plan.structure(),
                node.id.as_id(),
                &symbols,
            )
            .unwrap();
        units.push(CausalUnitRequest {
            node: node.id.as_id(),
            inputs,
            outputs,
            realization: CausalUnitRealization::Conditional {
                residuals: inventory.residuals,
                unknowns: inventory.unknowns,
                solver: Box::new(crate::math::settings::SolveSettings {
                    intent: SolveIntent::Root,
                    backend: Some(Backend::Kinsol),
                    ..Default::default()
                }),
            },
        });
    }
    assert_eq!(nodes.len(), 3);
    assert_eq!(model.connections.len(), 3);
    let tear = model
        .connections
        .values()
        .find(|c| c.lineage.path.ends_with(".recycle") || c.lineage.path.contains(".recycle["))
        .unwrap()
        .id;
    let selection = ModelingFlowSelection {
        nodes,
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.0,
                        policy: if *id == tear {
                            Policy::Mandatory
                        } else {
                            Policy::Forbidden
                        },
                    },
                )
            })
            .collect(),
    };
    let prepared = package
        .prepare_recycle(
            &analysis,
            selection,
            RecycleRequest {
                tears: BTreeSet::from([tear]),
                units,
                anderson: 0,
                damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(prepared.conditional_units().count(), 3);
    let expected_order: Vec<_> = prepared.request.units.iter().map(|u| u.node).collect();
    assert_eq!(prepared.order().unwrap(), expected_order);
    let report = prepared.start().unwrap().finish().await.unwrap();
    assert_eq!(
        report.report.qualification,
        Qualification::Feasible,
        "{:?}",
        report.report
    );
    let candidate = report.report.candidate.as_ref().unwrap();
    assert_eq!(candidate.primal.len(), 1);
    assert!((candidate.primal[0] - 1.).abs() < 1e-6);

    // Independently evaluate the original unit equations at the converged tear.
    // Feed/product closure is a boundary observation, not the fixed-point residual.
    let controls = Controls::default();
    let execution = Execution::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        &controls,
    );
    let mut boundary = BTreeMap::from([(ports["mixer.inlet"], candidate.primal[0])]);
    for program in prepared.programs.clone() {
        let original = program.values.identity();
        let budget = crate::math::WorkerBudget::new(16 << 20);
        let worker = package
            .runtime
            .native()
            .worker(
                program.program.clone(),
                &prepared.providers,
                execution.scope().unwrap(),
                &budget,
            )
            .unwrap();
        let mut unit = UnitWorker {
            last_values: None,
            input_ids: program.inputs.iter().map(|i| i.0).collect(),
            output_ids: program.outputs.iter().map(|o| o.0).collect(),
            program,
            worker,
            service: package.runtime.native().clone(),
            providers: prepared.providers.clone(),
            budget,
        };
        boundary.extend(unit.evaluate(&boundary, &execution).unwrap());
        assert_eq!(unit.program.values.identity(), original);
        if boundary.contains_key(&ports["mixer.outlet"]) {
            boundary.insert(ports["reactor.inlet"], boundary[&ports["mixer.outlet"]]);
        }
        if boundary.contains_key(&ports["reactor.outlet"]) {
            boundary.insert(ports["separator.inlet"], boundary[&ports["reactor.outlet"]]);
        }
    }
    assert!((boundary[&ports["mixer.outlet"]] - 2.).abs() < 1e-6);
    assert!((boundary[&ports["reactor.outlet"]] - 2.).abs() < 1e-6);
    assert!((1. - boundary[&ports["separator.outlet"]]).abs() < 1e-6);
    assert!((boundary[&ports["separator.recycle"]] - candidate.primal[0]).abs() < 1e-6);
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn indexed_mixer_holdup_separator_temporal_closure_is_independent_and_requires_initial_inventory()
 {
    let cancel = crate::CancelSource::new();
    for (source, closes) in [
        (SOURCE.to_owned(), true),
        (
            SOURCE.replace(
                "flux mixer[i].fresh-separator[i].product",
                "flux erroneous_flux",
            ),
            false,
        ),
    ] {
        let package = package(&source);
        let root = package
            .declarations()
            .iter()
            .find(|r| r.name == "dynamic")
            .unwrap()
            .declaration_id;
        let prepared = package
            .declared_simulation(root, compiler_profile(), None, Default::default(), &cancel)
            .await
            .unwrap();
        assert_eq!(prepared.model().compiled().model.connections.len(), 3);
        assert_eq!(prepared.contract().balances.len(), 1);
        let trajectory = prepared.run(&cancel).await.unwrap();
        assert_eq!(
            trajectory.report.termination,
            native::dynamics::Termination::Completed,
            "{:?}",
            trajectory.report.error
        );
        assert_eq!(
            trajectory.accepted, closes,
            "{:?}",
            trajectory.validation_error
        );
        let last = trajectory.report.conservation.last().unwrap();
        let expected = 2. - (-0.5_f64).exp();
        assert!((last.inventories[0] - expected).abs() < 1e-6);
        if closes {
            assert!(
                trajectory
                    .report
                    .conservation
                    .iter()
                    .all(|s| s.defects[0].abs() < 1e-6)
            );
            assert!((last.integrals[0] - (expected - 1.)).abs() < 1e-6);
        } else {
            assert!(last.defects[0].abs() > 0.3);
        }
    }
    let package = package(&SOURCE.replace(
        "eq initial:reactor[0{s}].inventory==starting_inventory;",
        "",
    ));
    let root = package
        .declarations()
        .iter()
        .find(|r| r.name == "dynamic")
        .unwrap()
        .declaration_id;
    let error = package
        .declared_simulation(root, compiler_profile(), None, Default::default(), &cancel)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("initial"), "{error}");
}
