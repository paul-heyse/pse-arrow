// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::workflow::{
    dynamics::tests as dynamic,
    tests::{compiler_profile, id},
};

#[tokio::test]
async fn nonzero_origin_smooth_scheduled_and_state_reset_fits_share_response_contract() {
    for mode in 0..3 {
        let mut builder =
            dynamic::source_with_runtime(crate::workflow::tests::runtime_with_workspace(32 << 20));
        builder.sources.dynamics[0].time_origin = Some(100.0);
        if mode == 2 {
            let neutral = builder
                .physical_context()
                .quantities
                .neutral_dimensionless()
                .unwrap();
            let scalar_unit = builder
                .physical_context()
                .quantities
                .quantity_type(neutral)
                .unwrap()
                .canonical_unit
                .as_id();
            let d = builder.declaration_mut();
            for (n, expression) in [(13, "state-initial*1.1"), (14, "initial*2")] {
                let mut definition = d.definitions[2].clone();
                definition.definition_id = id(n);
                definition.sources = vec![expression.into()];
                definition.units.push(
                    serde_json::from_value(
                        serde_json::json!({"spelling":"1","unit_id":scalar_unit}),
                    )
                    .unwrap(),
                );
                d.definitions.push(definition);
                let mut instance = d.cases[0].instances[2].clone();
                instance.instance_id = id(n + 20);
                instance.definition_id = id(n);
                instance.contributions[0].row_id = Some(id(n + 30));
                d.cases[0].instances.push(instance);
                let mut row = d.cases[0].rows[2].clone();
                row.row_id = id(n + 30);
                d.cases[0].rows.push(row);
            }
            let d = &mut builder.sources.dynamics[0];
            d.modes.push(d.modes[0].clone());
            d.modes[0].events.push(serde_json::from_value(serde_json::json!({"event_id":id(55),"guard_row":id(43),"reset_rows":[id(44)],"terminal":false,"next_mode":1,"tolerance":1e-8})).unwrap());
        }
        let time = builder
            .physical_context()
            .quantities
            .quantity_type(pse_quantity::QuantityTypeId::from_id(id(63)))
            .unwrap()
            .canonical_unit
            .as_id();
        let expected = match mode {
            0 => 74.0,
            1 => 73.0,
            _ => 83.0,
        };
        builder.dataset(serde_json::from_value(serde_json::json!({"dataset_id":id(70),"name":"offset clock","source":"analytic fixture","content_hash":ContentHash::from_bytes([1;32])})).unwrap());
        builder.observation(serde_json::from_value(serde_json::json!({"observation_id":id(71),"dataset_id":id(70),"target":"response","value":expected,"unit_id":time,"std_dev":1.0,"timestamp":null,"tag":null,"source_span":{"document_id":id(70),"start":0,"end":0}})).unwrap());
        builder.fit(serde_json::from_value(serde_json::json!({"fit_id":id(73),"model_id":id(20),"parameters":[{"symbol_id":id(3),"fixed":false,"value":2.0,"lower":1.1,"upper":10.0,"scale":1.0}],"experiments":[{"experiment_id":id(74),"case_id":id(5),"dynamic_id":id(50)}],"observations":[{"observation_id":id(71),"experiment_id":id(74),"output_id":id(42),"time":61.0,"time_basis":"model_clock","time_unit_id":time,"included":true,"importance":1.0}]})).unwrap());
        let mut simulation = dynamic::profile();
        simulation.start = 160.0;
        simulation.end = 161.0;
        simulation.samples = vec![160.0, 161.0];
        if mode == 1 {
            simulation.changes.push(native::dynamics::InputChange {
                time: 160.5,
                parameters: vec![1.0],
            });
        }
        let profile = FitProfile {
            solver: SolverProfile {
                intent: SolveIntent::Optimize,
                selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
                controls: native::solve::Controls {
                    hessian: HessianMode::LimitedMemory,
                    ..Default::default()
                },
                presolve: native::presolve::Policy::Off,
                numerics: Default::default(),
                convexity: Default::default(),
                backend: crate::math::solves::BackendSettings::Default,
            },
            simulations: BTreeMap::from([(id(74), simulation)]),
            rank_tolerance: 1e-8,
            max_cells: 100000,
        };
        let revision = builder.freeze().unwrap();
        let prepared = revision
            .prepare_fit_problem(
                id(73),
                profile.clone(),
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(prepared.measurements[0].time, Some(161.0));
        assert_eq!(prepared.measurements[0].sample_index, Some(0));
        let execution = Execution::new(
            Arc::new(AtomicBool::new(false)),
            &prepared.profile.solver.controls,
        );
        let mut oracle = FitOracle::new(prepared, execution).unwrap();
        let (response, _, rank) = oracle.response_rank(&[2.0]).unwrap();
        let derivative = if mode == 1 { 0.5 } else { 1.0 };
        assert!((response[(0, 0)] - derivative).abs() < 1e-5, "mode {mode}");
        assert_eq!(rank, 1);
        drop(oracle);
        let prepared = revision
            .prepare_fit(
                id(73),
                profile,
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        assert!(
            (report.candidate.as_ref().unwrap()[0] - 3.0).abs() < 1e-4,
            "mode {mode}: {report:?}"
        );
        assert!(report.responses.is_some());
        assert!(report.estimate_qualified(), "mode {mode}: {report:?}");
    }
}
