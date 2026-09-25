// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::{
    MathError,
    guarded::*,
    library::{self, Optimization},
};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use std::sync::Arc;
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
use symbolica::atom::{Atom, AtomCore};

fn compile_artifact(
    inputs: usize,
    slots: usize,
    outputs: Vec<usize>,
    stages: &[Stage],
    options: Optimization,
    cancel: &Arc<AtomicBool>,
) -> Result<Worker, MathError> {
    let order = if stages.iter().any(|s| matches!(s, Stage::Branch { .. })) {
        DerivativeOrder::Value
    } else {
        DerivativeOrder::Second
    };
    let body = PreparedBody::new(
        inputs,
        slots,
        outputs,
        stages.to_vec(),
        DerivativeOrder::Second,
    )?;
    body.compile(
        &(0..body.output_count()).collect::<Vec<_>>(),
        &(0..inputs).collect::<Vec<_>>(),
        order,
        options,
        crate::jets::EvaluationLimits::default(),
        cancel,
    )
    .map(|a| a.worker())
}
fn source() -> SemanticId {
    SemanticId::from_bytes([1; 16])
}
fn block(expr: Atom, output: Slot) -> Stage {
    Stage::Block {
        expressions: vec![expr],
        outputs: vec![output],
        source: source(),
    }
}
fn compile(inputs: usize, slots: usize, stages: &[Stage], output: Slot) -> Worker {
    compile_artifact(
        inputs,
        slots,
        vec![output],
        stages,
        Optimization::default(),
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap()
}
fn run(
    artifact: &mut Worker,
    inputs: &[f64],
    order: DerivativeOrder,
) -> Result<Vec<f64>, MathError> {
    artifact
        .evaluate(
            inputs,
            order,
            &mut BTreeMap::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .map(|r| r.values)
}

#[test]
fn domain_survives_symbolic_cancellation() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let normalized = &x / &x;
    assert_eq!(normalized, Atom::num(1));
    let mut artifact = compile(
        1,
        2,
        &[
            Stage::Require {
                argument: 0,
                condition: Condition::Nonzero,
                order: DerivativeOrder::Value,
                source: source(),
            },
            block(normalized, 1),
        ],
        1,
    );
    assert!(matches!(
        run(&mut artifact, &[0.0], DerivativeOrder::Value),
        Err(MathError::Domain { .. })
    ));
    assert_eq!(
        run(&mut artifact, &[2.0], DerivativeOrder::Value).unwrap(),
        vec![1.0]
    );
}

#[test]
fn lazy_branch_and_cloned_workers_do_not_evaluate_inactive_domain() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let zero = library::formal(1).unwrap();
    let positive = vec![
        Stage::Require {
            argument: 0,
            condition: Condition::Positive,
            order: DerivativeOrder::Value,
            source: source(),
        },
        block(x.log(), 2),
    ];
    let mut artifact = compile(
        1,
        3,
        &[
            block(Atom::num(0), 1),
            Stage::Branch {
                comparison: Comparison::Lt,
                left: 1,
                right: 0,
                then: positive,
                otherwise: vec![block(zero, 2)],
            },
        ],
        2,
    );
    let mut clone = artifact.clone();
    assert_eq!(
        run(&mut artifact, &[-1.0], DerivativeOrder::Value).unwrap(),
        vec![0.0]
    );
    assert_eq!(
        run(&mut clone, &[1.0], DerivativeOrder::Value).unwrap(),
        vec![0.0]
    );
    assert!(
        (run(&mut artifact, &[2.0], DerivativeOrder::Value).unwrap()[0] - 2.0_f64.ln()).abs()
            < 1e-14
    );
}

#[test]
fn square_root_boundary_has_separate_value_and_derivative_admission() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let mut artifact = compile(
        1,
        2,
        &[
            Stage::Require {
                argument: 0,
                condition: Condition::Nonnegative,
                order: DerivativeOrder::Value,
                source: source(),
            },
            Stage::Require {
                argument: 0,
                condition: Condition::Positive,
                order: DerivativeOrder::First,
                source: source(),
            },
            block(x.sqrt(), 1),
        ],
        1,
    );
    assert_eq!(
        run(&mut artifact, &[0.0], DerivativeOrder::Value).unwrap(),
        vec![0.0]
    );
    assert!(run(&mut artifact, &[0.0], DerivativeOrder::First).is_err());
    assert_eq!(
        run(&mut artifact, &[4.0], DerivativeOrder::Second).unwrap(),
        vec![2.0]
    );
    assert!(run(&mut artifact, &[-1.0], DerivativeOrder::Value).is_err());
    assert_eq!(
        run(&mut artifact, &[9.0], DerivativeOrder::Value).unwrap(),
        vec![3.0]
    );
}

#[test]
fn bounded_layout_and_cancellation_reject_before_execution() {
    crate::initialize().unwrap();
    let cancelled = Arc::new(AtomicBool::new(true));
    assert!(matches!(
        compile_artifact(
            0,
            1,
            vec![0],
            &[block(Atom::num(1), 0)],
            Optimization::default(),
            &cancelled
        ),
        Err(MathError::Cancelled)
    ));
    assert!(
        compile_artifact(
            0,
            library::MAX_FORMAL_SYMBOLS + 1,
            vec![0],
            &[],
            Optimization::default(),
            &Arc::new(AtomicBool::new(false))
        )
        .is_err()
    );
    let mut artifact = compile(1, 2, &[block(library::formal(0).unwrap(), 1)], 1);
    assert!(
        artifact
            .evaluate(
                &[2.0],
                DerivativeOrder::Value,
                &mut BTreeMap::new(),
                &cancelled
            )
            .is_err()
    );
    assert!(run(&mut artifact, &[f64::NAN], DerivativeOrder::Value).is_err());
    assert_eq!(
        run(&mut artifact, &[2.0], DerivativeOrder::Value).unwrap(),
        vec![2.0]
    );
}

#[test]
fn explicit_optimizer_thread_budget_uses_library_capability() {
    crate::initialize().unwrap();
    crate::initialize().unwrap();
    let capabilities = symbolica::license::LicenseManager::execution_capabilities();
    let cores = symbolica::license::LicenseManager::max_threads(2);
    assert!(cores > 0 && cores <= 2);
    if std::env::var_os("SYMBOLICA_LICENSE").is_some() {
        assert!(capabilities.is_licensed);
    }
    if capabilities.is_licensed {
        assert_eq!(cores, 2);
    }
    let x = library::formal(0).unwrap();
    let expression = (&x + Atom::num(1)).pow(Atom::num(3));
    let mut artifact = compile_artifact(
        1,
        2,
        vec![1],
        &[block(expression, 1)],
        Optimization {
            cores,
            ..Optimization::default()
        },
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(
        run(&mut artifact, &[2.0], DerivativeOrder::Value).unwrap(),
        vec![27.0]
    );
    eprintln!(
        "Symbolica capability: licensed={}, optimizer cores={cores}",
        capabilities.is_licensed
    );
}

#[test]
fn schedule_refuses_forward_reads_and_partial_branch_outputs() {
    crate::initialize().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let attempt = |stages: Vec<Stage>| {
        compile_artifact(1, 3, vec![2], &stages, Optimization::default(), &cancel)
    };
    assert!(
        attempt(vec![
            block(library::formal(1).unwrap(), 2),
            block(Atom::num(1), 1)
        ])
        .is_err()
    );
    assert!(
        attempt(vec![
            Stage::Require {
                argument: 1,
                condition: Condition::Positive,
                order: DerivativeOrder::Value,
                source: source()
            },
            block(Atom::num(1), 2)
        ])
        .is_err()
    );
    assert!(
        attempt(vec![Stage::Branch {
            comparison: Comparison::Eq,
            left: 0,
            right: 0,
            then: vec![block(Atom::num(1), 2)],
            otherwise: vec![]
        }])
        .is_err()
    );
}

#[test]
fn high_precision_real_values_survive_optimizer_and_jet_profiles() {
    crate::initialize().unwrap();
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/plan14/real-algebra-reference.json"
    ))
    .unwrap();
    assert_eq!(reference["precision"], 100);
    let x = library::formal(0).unwrap();
    for options in [
        Optimization::default(),
        Optimization {
            cores: symbolica::license::LicenseManager::max_threads(2),
            horner_iterations: 0,
            cpe_iterations: 0,
        },
    ] {
        let expression = (&x + Atom::num(1)).log();
        let mut worker = compile_artifact(
            1,
            2,
            vec![1],
            &[block(expression, 1)],
            options,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        for row in reference["cases"].as_array().unwrap() {
            let x = row["x"].as_f64().unwrap();
            let expected = row["log1px"].as_str().unwrap().parse::<f64>().unwrap();
            for order in [
                DerivativeOrder::Value,
                DerivativeOrder::First,
                DerivativeOrder::Second,
            ] {
                let actual = run(&mut worker, &[x], order).unwrap()[0];
                assert!(
                    (actual - expected).abs() <= 2e-12 * expected.abs().max(1.),
                    "{actual} versus Decimal {expected}"
                );
            }
        }
    }
}
