// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical unit/coordinate contract fixtures, separate from process qualification.
use super::*;
use crate::workflow::{
    ModelBuilder,
    tests::{compiler_profile, id, physical, runtime},
};
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
