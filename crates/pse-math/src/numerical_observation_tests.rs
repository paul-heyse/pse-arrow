// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    binding::{CaseStructure, ScalarInputInventory},
    construction::ConstructionAdmission,
    jets::EvaluationLimits,
    library::Optimization,
    typed::{Binary, BodyBuilder, BodyLimits},
};
use pse_model::generated::enums::ModelingVariableDomain;
use pse_quantity::{
    IndexSet,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::collections::BTreeMap;

#[derive(Debug)]
struct Construction;
impl ConstructionAdmission for Construction {
    fn try_grow(&self, _: usize) -> Result<(), MathError> {
        Ok(())
    }
}
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}

pub(super) fn transport_fixture(
    two: bool,
    order: DerivativeOrder,
) -> (
    Arc<NumericalObservationCase>,
    Arc<RestoredExecutionTemplate>,
) {
    let standard = standard_registry().unwrap();
    let temperature = ids::quantity("temperature.point");
    let difference = ids::quantity("temperature.difference");
    let neutral = ids::quantity("neutral");
    let canonical = standard.quantity_type(temperature).unwrap().canonical_unit;
    let mut declarations = standard.to_builder();
    let mut unit = standard.unit(canonical).unwrap().clone();
    unit.id = pse_quantity::UnitId::from_id(id(220));
    unit.symbol = "numerical-observation-temperature".into();
    unit.scale_to_canonical = 2.;
    unit.offset_to_canonical = 10.;
    unit.is_affine = true;
    unit.definition = None;
    let affine = unit.id;
    declarations.unit(unit);
    let mut unit = standard.unit(canonical).unwrap().clone();
    unit.id = pse_quantity::UnitId::from_id(id(221));
    unit.symbol = "numerical-observation-temperature-second".into();
    unit.scale_to_canonical = 3.;
    unit.offset_to_canonical = -7.;
    unit.is_affine = true;
    unit.definition = None;
    let second = unit.id;
    declarations.unit(unit);
    let registry = declarations.build().unwrap();
    let port = |n, unit| Port {
        id: id(n),
        quantity: temperature,
        unit,
    };
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &StandardInvariantChecker,
        3,
        BodyLimits::default(),
    )
    .unwrap();
    let a = builder
        .input(0, temperature, IndexSet::new(), id(30))
        .unwrap();
    let b = builder
        .input(1, temperature, IndexSet::new(), id(31))
        .unwrap();
    let c = builder
        .input(2, temperature, IndexSet::new(), id(32))
        .unwrap();
    let ac = builder
        .binary(Binary::Sub, a, c.clone(), None, id(33))
        .unwrap();
    let bc = builder.binary(Binary::Sub, b, c, None, id(34)).unwrap();
    let one = builder
        .literal(
            1.,
            registry.unit(canonical).unwrap(),
            pse_quantity::literal::LiteralContext::Explicit {
                quantity_type: difference,
            },
            id(35),
        )
        .unwrap();
    let ac = builder
        .binary(Binary::Div, ac, one.clone(), None, id(36))
        .unwrap();
    let bc = builder.binary(Binary::Div, bc, one, None, id(37)).unwrap();
    let d = builder.binary(Binary::Add, ac, bc, None, id(38)).unwrap();
    let square = builder
        .binary(Binary::Mul, d.clone(), d, None, id(39))
        .unwrap();
    let body = builder.prepare(&[square]).unwrap();
    let producer = ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
    let compiled = body
        .compile(
            &[0],
            &[2, 0, 1],
            order,
            Optimization::default(),
            EvaluationLimits::default(),
            &producer,
        )
        .unwrap();
    let captured = crate::transfer::capture(&compiled, &producer, |_| {
        Ok::<Arc<dyn AllocationOwner>, &str>(Arc::new(()))
    })
    .unwrap();
    let template = Arc::new(
        crate::transfer::restore(captured, producer.clone(), Arc::new(Construction), |_| {
            Ok::<Arc<dyn AllocationOwner>, &str>(Arc::new(()))
        })
        .unwrap()
        .into_template(),
    );
    drop(compiled);
    drop(body);
    producer.cancellation_control().cancel();
    let variable = |port| Variable {
        port,
        fixed: false,
        domain: ModelingVariableDomain::Continuous,
        lower: Some(0.),
        upper: if two { None } else { Some(100.) },
    };
    let slots = vec![
        SlotBinding::new(&port(1, affine), &port(1, canonical), &registry).unwrap(),
        SlotBinding::new(&port(1, affine), &port(1, canonical), &registry).unwrap(),
        SlotBinding::new(&port(2, canonical), &port(2, canonical), &registry).unwrap(),
    ];
    let instance = NumericalObservationInstance::new(
        ExecutionAttributionSnapshot {
            instance: id(9),
            checked_members: Default::default(),
            checked_sources: Default::default(),
            occurrences: Default::default(),
        },
        slots,
        vec![
            Contribution {
                output: 0,
                target: Target::Row(id(7)),
                scale: 1.5,
            },
            Contribution {
                output: 0,
                target: Target::Row(id(7)),
                scale: -0.5,
            },
        ],
        vec![0],
        vec![2, 0, 1],
    )
    .unwrap();
    let mut variables = vec![variable(port(1, affine)), variable(port(2, canonical))];
    let mut rows = vec![Row {
        id: id(7),
        quantity: neutral,
        lower: f64::NEG_INFINITY,
        upper: f64::INFINITY,
    }];
    let mut coordinates = vec![id(2), id(1)];
    let mut instances = vec![instance];
    if two {
        variables.push(variable(port(3, second)));
        coordinates.push(id(3));
        rows.push(Row {
            id: id(8),
            quantity: neutral,
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
        });
        let slots = vec![
            SlotBinding::new(&port(3, second), &port(3, canonical), &registry).unwrap(),
            SlotBinding::new(&port(3, second), &port(3, canonical), &registry).unwrap(),
            SlotBinding::new(&port(2, canonical), &port(2, canonical), &registry).unwrap(),
        ];
        instances.push(
            NumericalObservationInstance::new(
                ExecutionAttributionSnapshot {
                    instance: id(10),
                    checked_members: Default::default(),
                    checked_sources: Default::default(),
                    occurrences: Default::default(),
                },
                slots,
                vec![Contribution {
                    output: 0,
                    target: Target::Row(id(8)),
                    scale: 1.,
                }],
                vec![0],
                vec![2, 0, 1],
            )
            .unwrap(),
        );
    }
    let binding = Arc::new(
        NumericalObservationCase::new(
            variables,
            vec![],
            rows,
            coordinates,
            instances,
            order,
            &registry,
            AssemblyLimits::default(),
        )
        .unwrap(),
    );
    (binding, template)
}
#[test]
fn execution_only_transport_keeps_affine_values_and_alias_collapsed_second_partials() {
    let (binding, template) = transport_fixture(false, DerivativeOrder::Second);
    let assembly = binding.assemble(vec![template]).unwrap();
    let control = pse_diagnostics::Cancellation::default();
    let mut worker = assembly
        .worker_scoped(ExecutionScope::new(control, None), |_| {
            Ok::<Arc<dyn AllocationOwner>, &str>(Arc::new(()))
        })
        .unwrap();
    let value = worker
        .evaluate(&CaseValues {
            scalars: BTreeMap::from([(id(1), 1.), (id(2), 4.)]),
        })
        .unwrap();
    // d=(2x+10-y)+(2x+10-y)=16. d_y=-2,d_x=4.
    // f=d² gives H=[[8,-16],[-16,32]], including both local
    // off-diagonal alias terms in the single global x diagonal.
    assert_eq!(value.values, [256.]);
    assert_eq!(value.jacobian, [-64., 128.]);
    assert_eq!(value.hessians, [8., -16., -16., 32.]);
}

#[test]
fn symbolic_and_execution_only_inputs_share_fixed_domain_bounds_and_inventory_rules() {
    let registry = standard_registry().unwrap();
    let quantity = ids::quantity("neutral");
    let port = Port {
        id: id(1),
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    let variable = Variable {
        port,
        fixed: true,
        domain: ModelingVariableDomain::Integer,
        lower: Some(1.),
        upper: Some(4.),
    };
    let input = ScalarInputInventory::new(vec![variable.clone()], vec![], 10).unwrap();
    let ordinary = CaseStructure::new(
        vec![variable],
        vec![],
        vec![],
        vec![],
        None,
        CaseLimits::default(),
    )
    .unwrap();
    for (value, accepted) in [
        (2., true),
        (2.5, false),
        (0., false),
        (5., false),
        (f64::INFINITY, false),
    ] {
        let values = CaseValues {
            scalars: BTreeMap::from([(id(1), value)]),
        };
        assert_eq!(input.validate_values(&values).is_ok(), accepted);
        assert_eq!(ordinary.validate_values(&values).is_ok(), accepted);
    }
    assert!(input.validate_values(&CaseValues::default()).is_err());
    assert!(
        input
            .validate_values(&CaseValues {
                scalars: BTreeMap::from([(id(1), 2.), (id(2), 3.)])
            })
            .is_err()
    );
    assert!(input.validate_scalar(id(2), 3.).is_err());
}

#[cfg(feature = "compiled-evaluator")]
#[test]
fn source_free_x4_observation_population_preserves_two_instances_tails_and_failed_points() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    let (binding, template) = transport_fixture(true, DerivativeOrder::Second);
    let compiler_scope = ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
    let native = Arc::new(
        template
            .compile_x4(64 << 20, &compiler_scope, |bytes| {
                assert!(bytes >= 64 << 20);
                Ok::<Arc<dyn AllocationOwner>, &str>(Arc::new(()))
            })
            .unwrap(),
    );
    drop(template);
    compiler_scope.cancellation_control().cancel();
    let assembly = binding.assemble(vec![Arc::clone(&native), native]).unwrap();
    let mut worker = assembly
        .batch_worker_scoped(
            ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            17,
            |_| Ok::<Arc<dyn AllocationOwner>, &str>(Arc::new(())),
        )
        .unwrap();
    let mut points = (0_u32..17)
        .map(|i| {
            let i = f64::from(i);
            CaseValues {
                scalars: BTreeMap::from([(id(1), 1. + i), (id(2), 4. + i), (id(3), 2. + i)]),
            }
        })
        .collect::<Vec<_>>();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("source-free observation x4 transport");
    let entered = operation.context().enter();
    for count in [0, 1, 3, 4, 5, 7, 8, 9, 15, 16, 17] {
        let results = worker.evaluate(&points[..count]).unwrap();
        assert_eq!(results.len(), count);
        for (index, result) in results.iter().enumerate() {
            let i = f64::from(u32::try_from(index).unwrap());
            let d = 16. + 2. * i;
            let e = -10. + 4. * i;
            let result = result.as_ref().unwrap();
            assert_eq!(result.values, [d * d, e * e]);
            assert_eq!(result.jacobian, [-4. * d, 8. * d, 0., -4. * e, 0., 12. * e]);
            assert_eq!(
                result.hessians,
                [
                    8., -16., 0., -16., 32., 0., 0., 0., 0., 8., 0., -24., 0., 0., 0., -24., 0.,
                    72.
                ]
            );
        }
    }
    points[1].scalars.insert(id(1), 1e200);
    points[6].scalars.insert(id(3), 1e200);
    let results = worker.evaluate(&points).unwrap();
    assert!(
        matches!(results[1].as_ref().unwrap_err(),MathError::Instance{instance,..} if *instance==id(9))
    );
    assert!(
        matches!(results[6].as_ref().unwrap_err(),MathError::Instance{instance,..} if *instance==id(10))
    );
    assert!(
        results
            .iter()
            .enumerate()
            .all(|(i, result)| i == 1 || i == 6 || result.is_ok())
    );
    drop(entered);
    let snapshot = operation
        .reader()
        .snapshot(std::time::Duration::from_secs(60));
    assert!(
        snapshot
            .metrics
            .iter()
            .any(|metric| metric.name == "math.native_x4_packed_rows"
                && matches!(metric.cumulative,Value::Count(rows) if rows>0)),
        "the actual execution-only assembly must use native packed lanes"
    );
}

#[test]
fn execution_only_assembly_bound_and_owner_cover_escaping_case_and_workers() {
    let (binding, template) = transport_fixture(true, DerivativeOrder::Second);
    let templates = vec![Arc::clone(&template), template];
    binding.validate_templates(&templates).unwrap();
    let bound = binding.assembly_allocation_bytes(&templates).unwrap();
    let assembly = binding.assemble(templates).unwrap();
    assert!(
        bound >= assembly.retained_bytes(),
        "{bound} < {}",
        assembly.retained_bytes()
    );
    let owner = Arc::new(Construction);
    let weak = Arc::downgrade(&owner);
    let assembly = assembly.with_owner(owner.clone());
    drop(owner);
    drop(binding);
    let escaped = Arc::clone(assembly.binding());
    let worker = assembly
        .worker_scoped(
            ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            |_| Ok::<Arc<dyn AllocationOwner>, &str>(Arc::new(())),
        )
        .unwrap();
    drop(assembly);
    assert!(weak.upgrade().is_some());
    drop(worker);
    assert!(weak.upgrade().is_some());
    drop(escaped);
    assert!(weak.upgrade().is_none());
}
