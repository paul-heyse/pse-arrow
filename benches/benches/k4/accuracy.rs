// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same authored root with ordinary, separated-decision and refinement consumers.
use super::*;
#[path = "support.rs"]
mod support;
use pse_kernels::DerivativeOrder;
use pse_relations::{columnar::RelationRow, generated::runtime::accuracy_goal_assessments};
use pse_runtime::{math::solves::NumericalInputs, workflow::ModelingAnalysis};
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Write};

pub(super) fn measure(c: &mut Criterion, spec: &Value, output: &std::path::Path,
    phases: &phases::Phases) {
    let name = spec["id"].as_str().unwrap();
    let goal = match spec["consumer"].as_str().unwrap() {
        "ordinary" => "",
        "separated-decision" => "annotation accuracy_goal x(selected_output, steady, criterion_upper=10);",
        "refinement" => "annotation accuracy_goal x(selected_output, steady, resolution=0.1);",
        other => panic!("unknown accuracy consumer {other}"),
    };
    // This small scalar control isolates goal work. Its equation, initial point,
    // physical defaults and original acceptance remain identical in all three cases.
    let source = format!("package k4 {{ def Root {{ var x:Scalar; eq root:0.0001*(x*x-4)==0; annotation start x(1); annotation report x(\"x\"); {goal} }} }}");
    let sources = support::sources(&source);
    let executor = tokio::runtime::Builder::new_multi_thread().worker_threads(1)
        .enable_all().build().unwrap();
    let records_name = format!("{name}-observations.jsonl");
    let mut records = std::io::BufWriter::new(std::fs::File::create(output.join(&records_name)).unwrap());
    let mut maxima = BTreeMap::<&str, u64>::new();
    let mut observations = 0_u64;
    let mut group = c.benchmark_group("process");
    group.sample_size(10).sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(1)).measurement_time(Duration::from_secs(1));
    group.bench_function(name, |b| b.iter_custom(|iterations| {
        let mut elapsed_total = Duration::ZERO;
        for _ in 0..iterations {
            let (elapsed, record) = executor.block_on(async {
                let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(1).unwrap()).unwrap();
                let runtime = runtime(&owner);
                let begin = Instant::now();
                let package = support::package(&runtime, &owner, &sources).await;
                let root = package.declarations().iter().find(|row| row.name == "Root").unwrap().declaration_id;
                let admission = begin.elapsed();
                let mut solver = profile(Backend::Kinsol, false);
                solver.intent = pse_backend_native::solve::SolveIntent::Root;
                solver.presolve = pse_backend_native::presolve::Policy::Off;
                solver.controls.threads = 1;
                let analysis = ModelingAnalysis { root, instance: pse_modeling::specialize::root_instance(root),
                    bindings: Default::default(), limits: Default::default(), case: Default::default(),
                    order: DerivativeOrder::First, compiler: compiler(), solver,
                    numerical: NumericalInputs::default() };
                let before = owner.runtime.math().preparations();
                owner.runtime.reset_observation_peak();
                phases.reset();
                let begin = Instant::now();
                let cancel = CancelSource::new();
                let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
                let result = prepared.start().unwrap().wait().await.unwrap();
                let elapsed = begin.elapsed();
                let RunReport::Modeling(reports) = result.report().unwrap() else { panic!("wrong accuracy report"); };
                let mut numerical = observations::Observations::default();
                for (step, report) in reports.iter().enumerate() { numerical.modeling(report, step); }
                let report = reports.last().unwrap();
                let coordinate = report.prepared.model.case.compiled().plan.columns()[0];
                let value = report.values.scalars[&coordinate];
                let goals = accuracy_goal_assessments::Row::rows(
                    &result.table("runtime.accuracy_goal_assessments").unwrap()).unwrap();
                assert_eq!(goals.len(), usize::from(!goal.is_empty()));
                let assessment = result.assessments().last().unwrap();
                assert_eq!(assessment.validated, Some(true));
                assert_eq!(assessment.numerically_feasible, Some(true));
                assert!(assessment.permits_result && result.usable(), "accuracy control must retain an originally qualified usable result");
                for goal in &goals {
                    assert_eq!(goal.status, pse_model::generated::enums::AccuracyGoalStatus::Satisfied,
                        "named goal control must realize its declared requirement");
                }
                let observed = numerical.json();
                let attempts = observed["strategy_ledger"]["work"]["attempts"]["total"]
                    .as_u64().expect("actual attempt count must be observed for this control");
                if spec["consumer"] == "refinement" {
                    assert!(attempts > 1, "refinement control must actually execute another attempt");
                } else {
                    assert_eq!(attempts, 1, "ordinary and separated decisions must use one solve");
                }
                let pool = owner.runtime.pool();
                let mut record = json!({"source_admission_seconds":admission.as_secs_f64(),
                    "seconds":elapsed.as_secs_f64(), "usable":result.usable(),
                    "value":value, "independent_root":2.0, "observed_root_difference":(value-2.0).abs(),
                    "goals":goals, "assessments":result.assessments(),
                    "numerical_observations":observed,
                    "preparations":support::counts(before,owner.runtime.math().preparations()),
                    "compiler_phases":phases.report(1), "compiler_constructions":phases.constructions(),
                    "pool_peak_bytes":owner.runtime.observation_peak_bytes(),
                    "process_peak_rss_bytes":owner.runtime.report().unwrap().process_peak_rss_bytes,
                    "retained_runtime_bytes":pool.reserved()});
                drop(result); drop(prepared); drop(package); drop(runtime); drop(owner);
                tokio::task::yield_now().await;
                record["after_runtime_teardown_bytes"] = pool.reserved().into();
                assert_eq!(pool.reserved(), 0, "accuracy workload escaped runtime ownership");
                (elapsed, record)
            });
            elapsed_total += elapsed;
            for field in ["pool_peak_bytes", "process_peak_rss_bytes", "retained_runtime_bytes", "after_runtime_teardown_bytes"] {
                if let Some(value) = record[field].as_u64() {
                    maxima.entry(field).and_modify(|held| *held = (*held).max(value)).or_insert(value);
                }
            }
            serde_json::to_writer(&mut records, &record).unwrap();
            records.write_all(b"\n").unwrap();
            observations = observations.checked_add(1).unwrap();
        }
        elapsed_total
    }));
    group.finish();
    records.flush().unwrap();
    let maximum = |field: &str| maxima.get(field).copied();
    let record = json!({"id":name,"workload":spec,"source":source,"iterations":observations,
        "threads":1,"native_threads":1,"pool_limit_bytes":64_u64<<30,
        "pool_peak_bytes":maximum("pool_peak_bytes"),"process_peak_rss_bytes":maximum("process_peak_rss_bytes"),
        "retained_runtime_bytes":maximum("retained_runtime_bytes"),
        "after_case_teardown_bytes":maximum("after_runtime_teardown_bytes"),
        "after_retained_runtime_teardown_bytes":maximum("after_runtime_teardown_bytes"),
        "cache_state":"fresh runtime; package admission precedes each timed operation",
        "timed_scope":"analysis preparation, automatic goal execution/refinement, original checks and retained completion",
        "sampling":"10 flat Criterion samples; source admission and result observation outside timer",
        "process_rss_scope":"whole-process lifetime peak, including compiler/allocator and bounded observation streaming; runtime pool peak is the scoped measure",
        "scope":"scalar goal-work control; no production scale or general solver speed claim", "observations_file":records_name});
    std::fs::write(output.join(format!("{name}-memory.json")), serde_json::to_vec_pretty(&record).unwrap()).unwrap();
}
