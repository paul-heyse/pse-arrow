// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Connected physical execution, distinct from the reference model's honest root refusal.
use super::*;
use crate::workflow::tests::{compiler_profile, physical, runtime};
use pse_model::generated::enums::NumericalTarget;
use pse_structural::flowsheet::{Decision, Policy};

const SOURCE: &str = include_str!("../../../../../../tests/fixtures/indexed-recycle-closure.pse");

async fn package(source: &str) -> ModelingPackage {
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
    runtime()
        .modeling_package(declarations, physical)
        .await
        .unwrap()
}

#[tokio::test]
async fn indexed_mixer_holdup_separator_executes_conditional_recycle_and_external_closure() {
    let package = package(SOURCE).await;
    let cancel = crate::CancelSource::new();
    let root = package
        .declarations()
        .await
        .unwrap()
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
    let mut unit_instances = BTreeMap::new();
    for name in ["mixer", "reactor", "separator"] {
        let node = model
            .instances
            .values()
            .find(|n| {
                n.path.ends_with(&format!(".{name}")) || n.path.contains(&format!(".{name}["))
            })
            .unwrap();
        unit_instances.insert(name, node.id);
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
    let tear_target = crate::workflow::tests::engineering_target(
        prepared.numerics(),
        NumericalTarget::Variable,
        prepared.contract.variables[0].id,
    );
    let flow = package
        .quantities
        .quantity_type(tear_target.quantity.into())
        .unwrap();
    assert_eq!(flow.name.as_deref(), Some("Flow"));
    assert_eq!(flow.canonical_unit.as_id(), tear_target.unit);

    // Independently evaluate the original unit equations at the converged tear.
    // Feed/product closure is a boundary observation, not the fixed-point residual.
    let controls = Controls::default();
    let execution = Execution::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        &controls,
    );
    let mut boundary = BTreeMap::from([(ports["mixer.inlet"], candidate.primal[0])]);
    let mut original_values = BTreeMap::new();
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
        let mut values = unit.program.values.clone();
        values.scalars.extend(unit.original_values().scalars);
        let instance = model
            .instances
            .values()
            .find(|node| node.id.as_id() == unit.id())
            .unwrap();
        assert!(original_values.insert(instance.id, values).is_none());
        assert_eq!(unit.program.values.identity(), original);
        if boundary.contains_key(&ports["mixer.outlet"]) {
            boundary.insert(ports["reactor.inlet"], boundary[&ports["mixer.outlet"]]);
        }
        if boundary.contains_key(&ports["reactor.outlet"]) {
            boundary.insert(ports["separator.inlet"], boundary[&ports["reactor.outlet"]]);
        }
    }
    let value = |unit: &str, member: &str| {
        let instance = &model.instances[&unit_instances[unit]];
        let declaration = instance.members[member];
        let mut symbols = model.symbols.values().filter(|symbol| {
            symbol.lineage.instance == instance.id && symbol.lineage.declaration == declaration
        });
        let symbol = symbols.next().unwrap();
        assert!(symbols.next().is_none());
        original_values[&instance.id].scalars[&symbol.id]
    };
    let row_budget = |unit: &str, member: &str| {
        let instance = &model.instances[&unit_instances[unit]];
        let declaration = instance.members[member];
        let mut rows = model.equations.iter().filter(|row| {
            row.lineage.instance == instance.id && row.lineage.declaration == declaration
        });
        let row = rows.next().unwrap();
        assert!(rows.next().is_none());
        let target = crate::workflow::tests::engineering_target(
            &prepared.original_numerics,
            NumericalTarget::Row,
            row.id,
        );
        assert_eq!(target.quantity, tear_target.quantity);
        assert_eq!(target.unit, tear_target.unit);
        target.budget
    };
    let q = candidate.primal[0];
    let fresh = value("mixer", "fresh");
    assert_eq!(fresh, 1.);
    assert_eq!(value("mixer", "returned"), q);
    let mixed = value("mixer", "mixed");
    let reacted = value("reactor", "outgoing");
    let returned = value("separator", "returned");
    let product = value("separator", "product");
    assert_eq!(value("reactor", "incoming"), mixed);
    assert_eq!(value("separator", "incoming"), reacted);
    assert_eq!(mixed, boundary[&ports["mixer.outlet"]]);
    assert_eq!(reacted, boundary[&ports["reactor.outlet"]]);
    assert_eq!(returned, boundary[&ports["separator.recycle"]]);
    assert_eq!(product, boundary[&ports["separator.outlet"]]);
    let mixing = mixed - fresh - q;
    let balance = reacted - mixed;
    let split = returned - reacted / 2.;
    let discharge = reacted - value("reactor", "inventory") / value("reactor", "residence");
    let product_balance = product - reacted + returned;
    let mixing_budget = row_budget("mixer", "mixing");
    let balance_budget = row_budget("reactor", "balance");
    let split_budget = row_budget("separator", "split");
    let product_budget = row_budget("separator", "balance");
    assert!(mixing.abs() <= mixing_budget);
    assert!(balance.abs() <= balance_budget);
    assert!(split.abs() <= split_budget);
    assert!(discharge.abs() <= row_budget("reactor", "discharge"));
    assert!(product_balance.abs() <= product_budget);
    let defect = returned - q;
    let tear_row = crate::workflow::tests::engineering_target(
        prepared.numerics(),
        NumericalTarget::Row,
        prepared.contract.rows[0],
    );
    assert_eq!(tear_row.quantity, tear_target.quantity);
    assert_eq!(tear_row.unit, tear_target.unit);
    let tear_budget = tear_row.budget;
    assert_eq!(tear_budget, prepared.tolerances.rows[0]);
    assert!(defect.abs() <= tear_budget);
    // The exact map is F(q)=(1+q)/2. Its forward error also includes
    // the admitted errors of the individual conditional unit equations:
    // q-1 = -2*defect + mixing + balance + 2*split.
    let fixed_point_budget = 2. * tear_budget + mixing_budget + balance_budget + 2. * split_budget;
    assert!((q - fresh).abs() <= fixed_point_budget);
    assert!((mixed - 2. * fresh).abs() <= fixed_point_budget + mixing_budget);
    assert!((reacted - 2. * fresh).abs() <= fixed_point_budget + mixing_budget + balance_budget);
    // Product differs from fresh feed only by the recycle defect and the
    // original mixing, reactor and separator conservation residuals.
    assert!(
        (product - fresh).abs() <= tear_budget + mixing_budget + balance_budget + product_budget
    );
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
        let package = package(&source).await;
        let root = package
            .declarations()
            .await
            .unwrap()
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
        let inventory = prepared
            .numerics()
            .targets
            .iter()
            .find(|target| {
                target.kind == NumericalTarget::Variable
                    && package
                        .quantities
                        .quantity_type(target.quantity.into())
                        .unwrap()
                        .name
                        .as_deref()
                        == Some("TotalAmount")
            })
            .unwrap();
        let quantity = package
            .quantities
            .quantity_type(inventory.quantity.into())
            .unwrap();
        assert_eq!(quantity.canonical_unit.as_id(), inventory.unit);
        let inventory_budget = crate::workflow::tests::engineering_target(
            prepared.numerics(),
            NumericalTarget::Variable,
            inventory.id,
        )
        .budget;
        let closure_budget = prepared.contract().balances[0].tolerance;
        assert_eq!(closure_budget, inventory_budget);
        let fraction = prepared.numerics().policy.engineering_relative_fraction;
        assert_eq!(prepared.profile().rtol, fraction);
        assert!(
            prepared
                .profile()
                .atol
                .iter()
                .all(|value| *value == fraction)
        );
        assert_eq!(prepared.profile().out_atol, vec![closure_budget]);
        let trajectory = prepared.run(&cancel).await.unwrap();
        assert_eq!(
            trajectory.report().termination,
            native::dynamics::Termination::Completed,
            "{:?}",
            trajectory.report().error
        );
        assert_eq!(
            trajectory.accepted(),
            closes,
            "{:?}",
            trajectory.validation_error()
        );
        let last = trajectory.report().conservation.last().unwrap();
        let expected = 2. - (-0.5_f64).exp();
        assert!((last.inventories[0] - expected).abs() <= inventory_budget);
        if closes {
            assert!(
                trajectory
                    .report()
                    .conservation
                    .iter()
                    .all(|s| s.defects[0].abs() <= closure_budget)
            );
            assert!((last.integrals[0] - (expected - 1.)).abs() <= inventory_budget);
        } else {
            assert!(last.defects[0].abs() > 0.3);
        }
    }
    let package = package(&SOURCE.replace(
        "eq initial:reactor[0{s}].inventory==starting_inventory;",
        "",
    ))
    .await;
    let root = package
        .declarations()
        .await
        .unwrap()
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
