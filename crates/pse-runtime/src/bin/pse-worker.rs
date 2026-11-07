// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse-worker`: serve bounded canonical study actions on the supervised deployment.
//! Claims, cancellation and recovery use native fences. Scientific work stays outside
//! transactions, and completed native reports are retained as compact result selections.

// Match the library's depth for the shared runtime's nested async `Send` proof.
#![recursion_limit = "512"]
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a command-line worker reports on its own terminal"
)]

use std::process::ExitCode;
#[cfg(test)]
use std::{num::NonZeroUsize, time::Duration};

#[path = "support/native-stack.rs"]
mod native_stack;
#[path = "support/signals.rs"]
mod signals;

use clap::Parser;
#[cfg(test)]
use pse_runtime::workflow::WorkerSettings;
use pse_runtime::{CancelSource, workflow::WorkflowError};

#[path = "support/study-worker.rs"]
mod process;
use process::{Cli, ensure_openmp_environment, finish, runtime};
#[cfg(test)]
use process::{resource_budget, verify_placement_hierarchy};

fn diagnostic_report(error: &WorkflowError) -> String {
    format!("{:#?}", error.boundary_diagnostic())
}

fn report(error: &WorkflowError) {
    let code = miette::Diagnostic::code(error)
        .map(|code| format!(" [{code}]"))
        .unwrap_or_default();
    eprintln!("error{code}: {error}");
    eprintln!("diagnostic: {}", diagnostic_report(error));
    if let Some(help) = miette::Diagnostic::help(error) {
        eprintln!("help: {help}");
    }
}

#[cfg(test)]
fn worker_settings(
    cli: &Cli,
    allocation: pse_runtime::workflow::NativeAllocation,
) -> WorkerSettings {
    WorkerSettings {
        poll: cli
            .poll_ms
            .map(Duration::from_millis)
            .unwrap_or(WorkerSettings::default().poll),
        maximum_actions: cli.maximum_actions,
        maximum_in_flight: cli
            .maximum_in_flight
            .or(allocation.execution.map(|profile| profile.case_lanes)),
        until_idle: cli.until_idle,
        ..WorkerSettings::default()
    }
}

fn main() -> ExitCode {
    if let Err(error) = ensure_openmp_environment() {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    // block_on polls its root on the calling thread. RUST_MIN_STACK does not
    // resize the OS main stack: give the actual application root the same
    // qualified stack as native structural work, preserving a larger selection.
    let stack = match native_stack::selected() {
        Ok(stack) => stack,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    match std::thread::Builder::new()
        .name("pse-worker-root".into())
        .stack_size(stack)
        .spawn(move || run(stack))
    {
        Ok(root) => match root.join() {
            Ok(status) => status,
            Err(panic) => std::panic::resume_unwind(panic),
        },
        Err(error) => {
            eprintln!("error: worker root: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(stack: usize) -> ExitCode {
    if let Err(error) = ensure_openmp_environment() {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    let _telemetry_host =
        match pse_runtime::tooling::telemetry::Host::from_environment("pse-worker") {
            Ok(host) => host,
            Err(_) => {
                eprintln!("pse-telemetry: native-host-initialization-refused");
                None
            }
        };
    let cli = Cli::parse();
    let executor = match native_stack::runtime(stack) {
        Ok(executor) => executor,
        Err(error) => {
            eprintln!("error: executor: {error}");
            return ExitCode::FAILURE;
        }
    };
    let _executor_metrics = pse_runtime::tooling::runtime_metrics::register_executor(
        executor.handle(),
        pse_runtime::tooling::runtime_metrics::ExecutorRole::Worker,
    );
    let stop = CancelSource::new();
    // One original source spans startup, readiness, serving and native drain.
    let interrupt = match executor.block_on(async { signals::ProcessInterrupt::new(stop.clone()) })
    {
        Ok(interrupt) => interrupt,
        Err(error) => {
            eprintln!("error: worker stop listener: {error}");
            return ExitCode::FAILURE;
        }
    };
    let (runtime, selected) = match executor.block_on(runtime(&cli, &stop)) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("error: runtime initialization: {error}");
            return ExitCode::FAILURE;
        }
    };
    let diagnostics = runtime.observations().clone();
    let startup = selected.publish_ready(&runtime);
    let result = executor.block_on(async {
        let result = match startup {
            Ok(()) => runtime.serve(selected.settings(), &stop).await,
            Err(error) => {
                stop.cancel();
                Err(error)
            }
        };
        finish(&runtime, &selected, result).await
    });
    executor.block_on(interrupt.finish());
    // This process composition root joins its optional publisher after scientific
    // lanes have drained, outside RuntimeCaller admission and native ownership locks.
    diagnostics.stop_publisher();
    if let Some(error) = diagnostics.writer_failure() {
        eprintln!("diagnostic checkpoint unavailable: {error}");
    }
    match result {
        Ok(processed) => {
            println!("processed {processed} canonical scheduling actions");
            ExitCode::SUCCESS
        }
        Err(error) => {
            report(&error);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "assert the CLI parser and its production budget contract"
)]
mod tests {
    use super::*;

    #[test]
    fn workflow_error_diagnostic_report_retains_typed_detail_and_nested_cause() {
        use pse_model::diagnostic::{
            BoundaryClass, BoundaryDiagnostic, DiagnosticRule, DiagnosticStage,
        };

        let cause = pse_runtime::math::MathRuntimeError::Infrastructure(
            "specific native admission failure".into(),
        );
        let diagnostic = BoundaryDiagnostic::new(
            BoundaryClass::Infrastructure,
            DiagnosticStage::ModelingAdmission,
            [],
            DiagnosticRule::WorkflowUnclassified,
        )
        .with_code(pse_model::diagnostic::DiagnosticCode::RuntimeInfrastructure);
        let error = WorkflowError::ModelingAdmission {
            diagnostic: Box::new(diagnostic),
            cause: Box::new(cause),
        };

        let rendered = diagnostic_report(&error);
        assert!(rendered.contains("ModelingAdmission"), "{rendered}");
        assert!(rendered.contains("WorkflowUnclassified"), "{rendered}");
        assert!(
            rendered.contains("specific native admission failure"),
            "{rendered}"
        );
        assert!(rendered.contains("RuntimeInfrastructure"), "{rendered}");
    }
    fn placement_fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let parent = directory.path().join("reference.slice");
        let leaf = parent.join("primary.service");
        std::fs::create_dir_all(&leaf).unwrap();
        std::fs::write(leaf.join("memory.max"), (140_usize << 30).to_string()).unwrap();
        std::fs::write(leaf.join("cgroup.controllers"), "memory pids\n").unwrap();
        std::fs::write(parent.join("memory.max"), (160_usize << 30).to_string()).unwrap();
        std::fs::write(parent.join("cgroup.controllers"), "cpu memory pids\n").unwrap();
        std::fs::write(parent.join("cpu.max"), "1600000 100000\n").unwrap();
        (directory, parent, leaf)
    }

    #[test]
    fn managed_placement_inherits_exact_finite_cpu_quota_with_disabled_leaf_controller() {
        let (root, _, leaf) = placement_fixture();
        assert!(!leaf.join("cpu.max").exists());
        assert_eq!(
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16),
            Ok(())
        );
    }

    #[test]
    fn managed_placement_refuses_active_cpu_controller_without_quota_readback() {
        let (root, _, leaf) = placement_fixture();
        std::fs::write(leaf.join("cgroup.controllers"), "cpu memory pids\n").unwrap();
        let error =
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
        assert!(
            error.contains("active CPU controller lacks quota readback"),
            "{error}"
        );
    }

    #[test]
    fn managed_placement_refuses_narrower_cpu_ancestor_even_with_exact_leaf_quota() {
        let (root, parent, leaf) = placement_fixture();
        std::fs::write(leaf.join("cgroup.controllers"), "cpu memory pids\n").unwrap();
        std::fs::write(leaf.join("cpu.max"), "1600000 100000\n").unwrap();
        std::fs::write(parent.join("cpu.max"), "800000 100000\n").unwrap();
        let error =
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
        assert!(error.contains("ancestor CPU quota cannot admit"), "{error}");
    }

    #[test]
    fn managed_placement_refuses_narrower_memory_ancestor() {
        let (root, parent, leaf) = placement_fixture();
        std::fs::write(parent.join("memory.max"), (139_usize << 30).to_string()).unwrap();
        let error =
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
        assert!(
            error.contains("ancestor memory cap cannot admit"),
            "{error}"
        );
    }

    #[test]
    fn managed_placement_admits_wider_logical_cpu_envelope_than_scientific_width() {
        let (root, parent, leaf) = placement_fixture();
        std::fs::write(parent.join("cpu.max"), "3200000 100000\n").unwrap();
        assert_eq!(
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16),
            Ok(())
        );
    }

    #[test]
    fn managed_placement_requires_an_observed_finite_ancestor_cpu_quota() {
        {
            let quota = "max 100000\n";
            let (root, parent, leaf) = placement_fixture();
            std::fs::write(parent.join("cpu.max"), quota).unwrap();
            let error =
                verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
            assert!(
                error.contains("selected finite ancestor CPU quota was not observed"),
                "quota={quota:?}, error={error}"
            );
        }
        let (root, parent, leaf) = placement_fixture();
        std::fs::remove_file(parent.join("cpu.max")).unwrap();
        std::fs::write(parent.join("cgroup.controllers"), "memory pids\n").unwrap();
        let error =
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
        assert!(
            error.contains("selected finite ancestor CPU quota was not observed"),
            "{error}"
        );
    }

    fn legacy_allocation(bytes: usize) -> pse_runtime::workflow::NativeAllocation {
        pse_runtime::workflow::NativeAllocation::materialize(
            bytes.checked_mul(2).unwrap() + (512 << 20),
            512 << 20,
            2,
            bytes,
            None,
        )
        .unwrap()
    }
    #[test]
    fn selected_functional_worker_consumes_case_lanes_and_preserves_bounded_override() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let profiles = pse_runtime::workflow::ProductionProfiles::read(
            &root.join(".config/agent-capacity.toml"),
        )
        .unwrap();
        let allocation = profiles
            .select(pse_runtime::workflow::ProductionProfileName::Functional)
            .unwrap();
        let cli = Cli::try_parse_from(["pse-worker"]).unwrap();
        let selected = allocation.execution.unwrap();
        assert_eq!(selected.case_lanes, 2);
        assert_eq!(selected.math_jobs, 4);
        assert_eq!(
            worker_settings(&cli, allocation).maximum_in_flight,
            Some(selected.case_lanes)
        );
        let smaller = Cli::try_parse_from(["pse-worker", "--maximum-in-flight", "1"]).unwrap();
        assert!(resource_budget(&smaller, allocation).is_ok());
        assert_eq!(
            worker_settings(&smaller, allocation).maximum_in_flight,
            Some(1)
        );
        let larger = Cli::try_parse_from(["pse-worker", "--maximum-in-flight", "3"]).unwrap();
        assert!(resource_budget(&larger, allocation).is_err());
    }

    #[test]
    fn selected_reference_budget_consumes_exact_allocated_fields() {
        let cli = Cli::try_parse_from(["pse-worker"]).unwrap();
        let execution = pse_runtime::workflow::NativeExecutionAllocation {
            pool_memory_bytes: 128 << 30,
            worker_bytes: 16 << 30,
            cpu_threads: 16,
            case_lanes: 16,
            math_jobs: 32,
            compiler_cores: 1,
            admission_wait_ms: 30_000,
            process_headroom_bytes: 12 << 30,
            observer_memory_bytes: 4 << 30,
        };
        let allocation = pse_runtime::workflow::NativeAllocation::materialize(
            160 << 30,
            16 << 30,
            1,
            140 << 30,
            Some(execution),
        )
        .unwrap();
        let budget = resource_budget(&cli, allocation).unwrap();
        assert_eq!(budget.memory_limit_bytes.get(), 128 << 30);
        assert_eq!(budget.math.worker_bytes, 16 << 30);
        assert_eq!(budget.math.jobs, 32);
        assert_eq!(budget.threads.pool_threads.get(), 16);
        assert_eq!(
            budget.math.workspace_bytes,
            pse_runtime::math::MathPolicy::for_pool(execution.pool_memory_bytes).workspace_bytes
        );
        let cli = Cli::try_parse_from(["pse-worker", "--threads", "8"]).unwrap();
        assert!(resource_budget(&cli, allocation).is_err());
    }

    #[test]
    fn parsed_default_budget_admits_default_compiler_scratch() {
        let cli = Cli::try_parse_from(["pse-worker"]).unwrap();
        let budget = resource_budget(&cli, legacy_allocation(1 << 30)).unwrap();
        let available = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        assert_eq!(budget.threads.pool_threads.get(), available);
        assert_eq!(budget.threads.target_partitions.get(), available);
        let compiler = pse_runtime::workflow::PreparationSettings::default().compiler;
        assert!(budget.math.worker_bytes >= compiler.evaluation.scratch_bytes);
        let compilation = compiler.evaluation.scratch_bytes
            + budget.math.stack_bytes
            + budget.math.foreign_bytes
            + budget.math.inner_session_bytes;
        assert!(compilation + budget.math.workspace_bytes <= budget.memory_limit_bytes.get());
    }

    #[test]
    fn explicit_smaller_budget_retains_its_compilation_admission_limit() {
        let cli = Cli::try_parse_from(["pse-worker", "--memory-mib", "512"]).unwrap();
        let budget = resource_budget(&cli, legacy_allocation(1 << 30)).unwrap();
        assert!(
            budget.math.worker_bytes
                < pse_runtime::workflow::PreparationSettings::default()
                    .compiler
                    .evaluation
                    .scratch_bytes
        );
    }

    #[test]
    fn explicit_worker_budget_cannot_exceed_the_supervised_slot() {
        let cli = Cli::try_parse_from(["pse-worker", "--memory-mib", "2048"]).unwrap();
        assert!(resource_budget(&cli, legacy_allocation(1 << 30)).is_err());
    }
}
