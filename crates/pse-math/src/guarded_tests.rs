// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::{
    MathError,
    guarded::*,
    library::{self, Optimization},
};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use symbolica::atom::{Atom, AtomCore};

fn compile_artifact(
    inputs: usize,
    slots: usize,
    outputs: Vec<usize>,
    stages: &[Stage],
    options: Optimization,
    cancel: &pse_diagnostics::Cancellation,
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
        &pse_kernels::ExecutionScope::new(cancel.clone(), None),
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
        &pse_diagnostics::Cancellation::new(false),
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
            &mut pse_kernels::ProviderBindings::new(),
            &pse_diagnostics::Cancellation::new(false),
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
                lineage: None,
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
            lineage: None,
        },
        block(x.log(), 2),
    ];
    let mut artifact = compile(
        1,
        3,
        &[
            block(Atom::num(0), 1),
            Stage::Branch {
                continuity: DerivativeOrder::Value,
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
                lineage: None,
            },
            Stage::Require {
                argument: 0,
                condition: Condition::Positive,
                order: DerivativeOrder::First,
                source: source(),
                lineage: None,
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
    let cancelled = pse_diagnostics::Cancellation::new(true);
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
    let mut artifact = compile(1, 2, &[block(library::formal(0).unwrap(), 1)], 1);
    assert!(
        artifact
            .evaluate(
                &[2.0],
                DerivativeOrder::Value,
                &mut pse_kernels::ProviderBindings::new(),
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
        &pse_diagnostics::Cancellation::new(false),
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
fn symbolic_initialization_preserves_fresh_and_embedding_global_pool_contracts() {
    // Symbolica and Rayon initialization are process-global. Each combination must
    // start in its own process, rather than inherit another test's initialized state.
    // Configured cases exercise this installation's actual capability; installations
    // without a key still exercise every child, rather than skip the whole control.
    for case in [
        "configured/fresh",
        "configured/embedding",
        "unlicensed/fresh",
        "unlicensed/embedding",
    ] {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child
            .args([
                "--exact",
                "guarded_tests::symbolic_initialization_global_pool_child",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("PSE_SYMBOLIC_INITIALIZATION_CONTROL", case)
            .env("SYMBOLICA_HIDE_BANNER", "1");
        if case.starts_with("unlicensed/") {
            child.env_remove("SYMBOLICA_LICENSE");
        }
        let output = child.output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            output.status.success(),
            "{case}: stdout={stdout} stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let marker = format!("SYMBOLIC_INITIALIZATION_CONTROL:{case}:");
        let outcome = stdout
            .lines()
            .find_map(|line| line.find(&marker).map(|start| &line[start..]))
            .unwrap_or_else(|| panic!("{case}: child did not execute its exact control"));
        eprintln!("{outcome}");
    }
}

#[test]
fn symbolic_initialization_global_pool_child() {
    let Ok(case) = std::env::var("PSE_SYMBOLIC_INITIALIZATION_CONTROL") else {
        return;
    };
    let embedding = case.ends_with("/embedding");
    let licensed =
        case.starts_with("configured/") && std::env::var_os("SYMBOLICA_LICENSE").is_some();
    if embedding {
        rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build_global()
            .unwrap();
    }
    crate::initialize().unwrap();
    let capabilities = symbolica::license::LicenseManager::execution_capabilities();
    assert_eq!(capabilities.is_licensed, licensed);
    let cores = symbolica::license::LicenseManager::max_threads(2);
    assert_eq!(cores, if licensed { 2 } else { 1 });
    let x = library::formal(0).unwrap();
    let mut artifact = compile_artifact(
        1,
        2,
        vec![1],
        &[block((x + Atom::num(1)).pow(3), 1)],
        Optimization {
            cores,
            ..Default::default()
        },
        &pse_diagnostics::Cancellation::default(),
    )
    .unwrap();
    assert_eq!(
        run(&mut artifact, &[2.], DerivativeOrder::Second).unwrap(),
        [27.]
    );
    // A late successful installation proves configured initialization/construction
    // did not eagerly create a project-owned global pool. The unlicensed library
    // fallback creates one thread only when no embedding host already owns it.
    let late = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build_global();
    assert_eq!(late.is_ok(), licensed && !embedding);
    let global_threads = rayon::current_num_threads();
    assert_eq!(global_threads, if licensed || embedding { 2 } else { 1 });
    // The parent detects exact-filter mistakes from stdout, independently of status.
    println!(
        "SYMBOLIC_INITIALIZATION_CONTROL:{case}:licensed={licensed};optimizer_cores={cores};global_threads={global_threads}"
    );
}

#[test]
fn schedule_refuses_forward_reads_and_partial_branch_outputs() {
    crate::initialize().unwrap();
    let cancel = pse_diagnostics::Cancellation::new(false);
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
                source: source(),
                lineage: None,
            },
            block(Atom::num(1), 2)
        ])
        .is_err()
    );
    assert!(
        attempt(vec![Stage::Branch {
            continuity: DerivativeOrder::Value,
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
            evaluator: library::Evaluator::Interpreted,
        },
    ] {
        let expression = (&x + Atom::num(1)).log();
        let mut worker = compile_artifact(
            1,
            2,
            vec![1],
            &[block(expression, 1)],
            options,
            &pse_diagnostics::Cancellation::new(false),
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

#[test]
fn branch_local_jets_refuse_unproved_boundaries_without_global_smoothness() {
    crate::initialize().unwrap();
    let x = library::formal(0).unwrap();
    let body = PreparedBody::new(
        1,
        3,
        vec![2],
        vec![
            block(Atom::num(0), 1),
            Stage::Branch {
                continuity: DerivativeOrder::Value,
                comparison: Comparison::Lt,
                left: 0,
                right: 1,
                then: vec![block(-x.clone(), 2)],
                otherwise: vec![block(x, 2)],
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    let control = pse_diagnostics::Cancellation::new(false);
    let cancel = control.flag();
    let limits = crate::jets::EvaluationLimits::default();
    assert_eq!(body.available_order(), DerivativeOrder::Value);
    assert!(
        body.compile(
            &[0],
            &[0],
            DerivativeOrder::First,
            Optimization::default(),
            limits,
            &pse_kernels::ExecutionScope::new(control.clone(), None)
        )
        .is_err()
    );
    let direct = body
        .compile_branch_local(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            limits,
            &pse_kernels::ExecutionScope::new(control.clone(), None),
        )
        .unwrap();
    let support = body
        .prepare_support(&[0], &[0], DerivativeOrder::Second, &cancel)
        .unwrap();
    let scope = pse_kernels::ExecutionScope::new(control.clone(), None);
    let scalars = support
        .prepare_scalars(Optimization::default(), limits, &scope)
        .unwrap();
    let reused = support
        .compile_branch_local_reusing(Optimization::default(), limits, &scope, &scalars)
        .unwrap();
    for compiled in [direct, reused] {
        let mut worker = compiled.worker();
        for (x, slope) in [(-2.0, -1.0), (2.0, 1.0)] {
            let jet = worker
                .evaluate(
                    &[x],
                    DerivativeOrder::Second,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel,
                )
                .unwrap();
            assert_eq!(jet.values, vec![2.0]);
            assert_eq!(jet.jacobian, vec![slope]);
            assert_eq!(jet.hessians, vec![0.0]);
        }
        assert!(
            worker
                .evaluate(
                    &[0.0],
                    DerivativeOrder::First,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel
                )
                .is_err()
        );
        assert_eq!(
            worker
                .evaluate(
                    &[0.0],
                    DerivativeOrder::Value,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel
                )
                .unwrap()
                .values,
            vec![0.0]
        );
    }
}
