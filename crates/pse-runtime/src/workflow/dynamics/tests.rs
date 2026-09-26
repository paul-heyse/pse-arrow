// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical unit/coordinate contract fixtures, separate from process qualification.
use super::*;
use crate::workflow::{
    ModelBuilder,
    tests::{compiler_profile, id, physical, runtime},
};
#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn dynamic_derivative_reuse_covers_all_varying_inputs_and_cancellation() {
    let mut source = source();
    source.declaration_mut().definitions[0].sources[0] = "parameter/parameter".into();
    let mut declaration = source.sources.dynamics[0].clone();
    declaration.modes.push(declaration.modes[0].clone());
    source.sources.dynamics[0] = declaration;
    let prepared = source
        .freeze()
        .unwrap()
        .prepare_simulation(
            id(50),
            profile(),
            compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let mut reused = prepared.worker(cancel.clone()).unwrap();
    let mut clean = prepared.worker(Arc::new(AtomicBool::new(false))).unwrap();
    let a = reused
        .evaluate(0, Function::Rhs, 0.0, &[0.0], &[2.0], true)
        .unwrap();
    for _ in 0..4 {
        let b = reused
            .evaluate(0, Function::Rhs, 0.0, &[0.0], &[2.0], true)
            .unwrap();
        assert_eq!(a.values, b.values);
        assert_eq!(
            a.jacobian.as_ref().unwrap().val(),
            b.jacobian.as_ref().unwrap().val()
        );
    }
    assert_eq!(reused.functions[&(0, Function::Rhs)].evaluations, 1);
    for (time, state, parameter) in [(1.0, 0.0, 2.0), (1.0, 1.0, 2.0), (1.0, 1.0, 3.0)] {
        let a = reused
            .evaluate(0, Function::Rhs, time, &[state], &[parameter], true)
            .unwrap();
        let b = clean
            .evaluate(0, Function::Rhs, time, &[state], &[parameter], true)
            .unwrap();
        assert_eq!(a.values, b.values);
        assert_eq!(a.jacobian.unwrap().val(), b.jacobian.unwrap().val());
    }
    assert_eq!(reused.functions[&(0, Function::Rhs)].evaluations, 4);
    reused
        .evaluate(1, Function::Rhs, 1.0, &[1.0], &[3.0], true)
        .unwrap();
    assert_eq!(reused.functions[&(1, Function::Rhs)].evaluations, 1);
    assert!(
        reused
            .evaluate(0, Function::Rhs, 1.0, &[1.0], &[0.0], true)
            .is_err()
    );
    assert!(reused.functions[&(0, Function::Rhs)].cache.is_none());
    reused
        .evaluate(0, Function::Rhs, 1.0, &[1.0], &[3.0], true)
        .unwrap();
    cancel.store(true, std::sync::atomic::Ordering::Release);
    assert!(matches!(
        reused.evaluate(0, Function::Rhs, 1.0, &[1.0], &[3.0], true),
        Err(ProblemError::Math(pse_math::MathError::Cancelled))
    ));
}
pub(in crate::workflow) fn source() -> ModelBuilder {
    source_with_runtime(runtime())
}
pub(in crate::workflow) fn source_with_runtime(runtime: crate::workflow::Runtime) -> ModelBuilder {
    use pse_quantity::*;
    let mut physical = physical();
    let q = physical.quantities.as_ref();
    let neutral = q
        .quantity_type(standard::ids::quantity("neutral"))
        .unwrap()
        .clone();
    let mut builder = q.to_builder();
    let unit = q
        .units()
        .find(|u| {
            u.dimension == DimensionVector::base(BaseDimension::Time) && u.scale_to_canonical == 1.0
        })
        .unwrap()
        .clone();
    let mut minute = unit.clone();
    minute.id = UnitId::from_id(id(61));
    minute.symbol = "fixture_minute".into();
    minute.scale_to_canonical = 60.0;
    builder.unit(minute.clone());
    let mut kind = q.kind(neutral.key.kind).unwrap().clone();
    kind.id = QuantityKindId::from_id(id(62));
    kind.dimension = unit.dimension;
    builder.kind(kind.clone());
    let mut time = neutral.clone();
    time.id = QuantityTypeId::from_id(id(63));
    time.key.kind = kind.id;
    time.canonical_unit = unit.id;
    builder.quantity_type(time.clone());
    builder.operation(QuantityOperation {
        id: OperationId::from_id(id(64)),
        opcode: Opcode::Derivative,
        input_kinds: vec![kind.id],
        result_kind: neutral.key.kind,
        basis_rule: BasisRule::Preserve,
        reference_rule: ReferenceRule::Preserve,
        scale_rule: QuantityScaleRule::Point,
        shape_rule: QuantityShapeRule::Scalar,
        basis_source: Some(0),
        reference_source: Some(0),
        scale_source: None,
        shape_source: None,
        subject_rule: SubjectRule::Preserve,
        subject_source: Some(0),
        result_subject_kind: None,
        result_basis: None,
        result_reference_state: None,
        input_conversions: vec![],
        precondition_invariants: vec![],
    });
    physical.quantities = Arc::new(builder.build().unwrap());
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let ports = [
        (id(1), time.id, minute.id),
        (id(2), time.id, minute.id),
        (id(3), neutral.id, neutral.canonical_unit),
        (id(4), time.id, unit.id),
    ];
    let definitions=[("rate","parameter",neutral.id),("initial","initial",time.id),("output","state + time",time.id)].into_iter().enumerate().map(|(i,(_,expr,_))|serde_json::json!({"definition_id":id(10+i as u8),"sources":[expr],"formals":ports.iter().zip(["state","time","parameter","initial"]).map(|((_,q,_),path)|serde_json::json!({"path":path,"quantity_id":q.as_id()})).collect::<Vec<_>>(),"domains":[],"groups":[],"providers":[],"units":[],"literals":[]})).collect::<Vec<_>>();
    let slots=ports.iter().map(|(s,q,_)|serde_json::json!({"source_id":s,"formal_quantity_id":q.as_id(),"formal_unit_id":physical.quantities.quantity_type(*q).unwrap().canonical_unit.as_id()})).collect::<Vec<_>>();
    let row:crate::workflow::ModelDeclaration=serde_json::from_value(serde_json::json!({"model_id":id(20),"name":"unit-coordinate control","domains":[],"groups":[],"definitions":definitions,"cases":[{"case_id":id(5),"name":"functions","variables":[{"port":{"symbol_id":id(1),"quantity_id":time.id.as_id(),"unit_id":minute.id.as_id()},"fixed":false,"domain":"continuous","lower":null,"upper":null}],"parameters":ports[1..].iter().map(|(s,q,u)|serde_json::json!({"symbol_id":s,"quantity_id":q.as_id(),"unit_id":u.as_id()})).collect::<Vec<_>>(),"instances":(0..3).map(|i|serde_json::json!({"instance_id":id(30+i),"definition_id":id(10+i),"slots":slots,"contributions":[{"output":0,"row_id":id(40+i),"scale":1.0}]})).collect::<Vec<_>>(),"rows":[{"row_id":id(40),"quantity_id":neutral.id.as_id(),"lower":null,"upper":null},{"row_id":id(41),"quantity_id":time.id.as_id(),"lower":null,"upper":null},{"row_id":id(42),"quantity_id":time.id.as_id(),"lower":null,"upper":null}],"objective":null,"values":[{"symbol_id":id(1),"value":0.0},{"symbol_id":id(2),"value":0.0},{"symbol_id":id(3),"value":2.0},{"symbol_id":id(4),"value":10.0}]}]})).unwrap();
    let mut b = ModelBuilder::from_declaration(runtime, row, physical);
    b.dynamics(serde_json::from_value(serde_json::json!({"dynamic_id":id(50),"model_id":id(20),"case_id":id(5),"time_id":id(2),"states":[{"symbol_id":id(1),"differential":true,"initial_row":id(41),"offset":10.0,"scale":2.0,"residual_scale":1.0}],"parameters":[id(3)],"outputs":[id(42)],"modes":[{"rhs_rows":[id(40)],"events":[]}]})).unwrap());
    b
}
pub(in crate::workflow) fn profile() -> SimulationProfile {
    SimulationProfile {
        samples: vec![0.0, 0.5, 1.0],
        sensitivities: true,
        parameter_scales: vec![1.0],
        ..Default::default()
    }
}
#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn compiler_coordinate_units_and_run_tables_preserve_physical_values() {
    let r = source().freeze().unwrap();
    let p = r
        .prepare_simulation(
            id(50),
            profile(),
            compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let handle = p.start().unwrap();
    let result = handle.wait().await.unwrap();
    let crate::workflow::RunReport::Simulation(report) = result.report().unwrap() else {
        panic!()
    };
    assert_eq!(
        report.termination,
        native::Termination::Completed,
        "{:?}",
        report.error
    );
    for sample in &report.samples {
        assert!((sample.state[0] - sample.time).abs() < 1e-6);
        assert!((sample.outputs[0] - (10.0 + 3.0 * sample.time)).abs() < 1e-6);
        assert!((sample.output_sensitivities[0] - sample.time).abs() < 1e-6);
    }
    assert_eq!(
        result
            .table("runtime.simulation_samples")
            .unwrap()
            .batch()
            .num_rows(),
        6
    );
    assert_eq!(
        result
            .table("authored.dynamic_cases")
            .unwrap()
            .batch()
            .num_rows(),
        1
    );
    assert!(!handle.progress().0.is_empty());
}
#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn initial_state_read_is_refused_before_execution() {
    let mut b = source();
    b.declaration_mut().definitions[1].sources[0] = "state".into();
    let r = b.freeze().unwrap();
    let error = r
        .prepare_simulation(
            id(50),
            profile(),
            compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("initial values may depend on parameters/time, not state guesses")
    );
}

#[cfg(not(feature = "solver-diffsol"))]
#[tokio::test]
async fn simulation_requires_a_linked_adapter() {
    let error = source()
        .freeze()
        .unwrap()
        .prepare_simulation(
            id(50),
            profile(),
            compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(error, WorkflowError::Contract(ref message) if message == "Diffsol adapter is not linked")
    );
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn checked_dynamic_rebind_matches_clean_preparation_and_shares_immutable_clones() {
    let cancel = crate::CancelSource::new();
    let revision = source().freeze().unwrap();
    let prepared = revision
        .prepare_simulation(id(50), profile(), compiler_profile(), &cancel)
        .await
        .unwrap();
    assert!(Arc::ptr_eq(&prepared.programs, &prepared.clone().programs));
    let mut selected = profile();
    selected.samples = vec![0.0, 0.25, 0.5];
    let rebound = prepared
        .rebind(
            &BTreeMap::from([(id(3), 4.0)]),
            selected.clone(),
            compiler_profile(),
            &cancel,
        )
        .await
        .unwrap();
    let mut clean = revision.edit();
    clean.declaration_mut().cases[0]
        .values
        .iter_mut()
        .find(|value| value.symbol_id == id(3))
        .unwrap()
        .value = 4.0;
    let clean = clean
        .freeze()
        .unwrap()
        .prepare_simulation(id(50), selected, compiler_profile(), &cancel)
        .await
        .unwrap();
    assert_eq!(rebound.identity(), clean.identity());
    assert_eq!(rebound.parameters, clean.parameters);
    assert_ne!(prepared.identity(), rebound.identity());
    assert!(
        prepared
            .rebind(
                &BTreeMap::from([(id(99), 1.0)]),
                profile(),
                compiler_profile(),
                &cancel
            )
            .await
            .is_err()
    );
    assert!(
        prepared
            .rebind(
                &BTreeMap::from([(id(3), f64::NAN)]),
                profile(),
                compiler_profile(),
                &cancel
            )
            .await
            .is_err()
    );
}
