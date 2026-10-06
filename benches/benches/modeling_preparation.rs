// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! M1 preparation scaling (Plan 23 H2): specialization, lowering and derivative programs of
//! the seed PC-SAFT homogeneous state at N components and of the Peng–Robinson density root
//! under its inline and nested realizations. Nothing is solved.
//!
//! Workloads are registered in `.config/preparation-cases.json`; `just bench-smoke` tests the
//! `smoke` ones once. `PSE_PREPARATION_CASE` selects one workload (one case per process gives
//! a per-case peak RSS) and `PSE_PREPARATION_OUTPUT` names a directory for the per-case
//! records, which are also printed. Each iteration admits a fresh runtime and seed closure
//! untimed, then measures cold, warm, value and structural preparations; the Symbolica formal
//! pool persists. The timed path is solver-independent, so its derivative programs are
//! first order; the nested realization needs the KINSOL capability (`native-process`), and
//! without it that workload records an infrastructure refusal. A typed resource refusal,
//! such as the formal-slot allowance, is a recorded result.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "a standalone measurement harness fails on invalid setup and emits its records"
)]
#[path = "k4/preparation.rs"]
mod k4_preparation;
#[path = "native_process/phases.rs"]
mod phases;
#[allow(
    dead_code,
    reason = "shared test runtime exposes constructors unused by this benchmark"
)]
#[path = "../../tests/support/workflow_runtime.rs"]
mod workflow_runtime;

use criterion::{Criterion, criterion_group, criterion_main};
use pse_runtime::{
    CancelSource,
    workflow::{ModelingPackage, Runtime, WorkflowError},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use workflow_runtime::WorkflowRuntime;

/// The bench owns only finite subsets and state cases of the published bank.
const PACKAGE: &str = "d4afeb91b5874b5087403590c363a6c1";
/// Seed state of the homogeneous PC-SAFT fixtures: 350 K and 1 bar.
const TEMPERATURE: &str = "350{K}";
const PRESSURE: &str = "100000{Pa}";

fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

/// The seed reference closure, loaded through the declared package document paths, with the
/// bench package from its authored texts.
async fn seed(owner: &WorkflowRuntime, bench: &BTreeMap<String, String>) -> ModelingPackage {
    use pse_runtime::authoring_driver::document::{
        load_bundles_owned, load_package, load_package_documents,
    };
    let root = repository().join("packages/reference");
    let pool = owner.runtime.pool();
    let validation = owner.sessions.validation_context(&owner.registry).unwrap();
    let documents = |names: &[&str], bench: Option<&BTreeMap<String, String>>| {
        let mut bundles = names
            .iter()
            .map(|name| {
                load_package(
                    &root.join(name),
                    &owner.registry,
                    Default::default(),
                    &validation,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        if let Some(texts) = bench {
            bundles.push(
                load_package_documents(
                    texts
                        .iter()
                        .map(|(path, text)| (path.clone(), text.as_bytes().to_vec()))
                        .collect(),
                    &owner.registry,
                    Default::default(),
                    &validation,
                )
                .unwrap(),
            );
        }
        load_bundles_owned(&bundles, &owner.registry, &pool, &owner.cancel, &validation).unwrap()
    };
    let runtime = Runtime::from_shared(
        owner.runtime.clone(),
        owner.registry.clone(),
        owner.sessions.clone(),
        pse_runtime::workflow::CanonicalDeployment::new(
            pse_operations::testing::canonical_fixture_store().unwrap(),
            pse_runtime::workflow::OuterAttestation {
                source: pse_ids::ContentHash::from_bytes([0; 32]),
                build: pse_ids::ContentHash::from_bytes([1; 32]),
            },
            None,
        ),
    );
    let physical = runtime
        .physical_from_documents(&documents(&["physical"], None), &owner.cancel)
        .await
        .unwrap();
    runtime
        .modeling_from_documents(
            &documents(
                &[
                    "data/oracles/teqp-0.23.1",
                    "data/oracles/feos-0.10.1",
                    "data/gross-sadowski-2001",
                    "data/references",
                    "data/nist",
                    "data/perry7",
                    "data/poling2000",
                    "data/oracles/idaes-2.13",
                    "data/species",
                    "data/ciaaw",
                    "seed-data",
                    "process",
                    "thermodynamics",
                    "methods",
                    "domain",
                    "physical",
                ],
                Some(bench),
            ),
            physical,
        )
        .await
        .unwrap()
}

/// Ordered membership of the published bank, read from its authored set declaration.
fn bank_members(owner: &WorkflowRuntime) -> Vec<String> {
    use pse_authoring::language::{StaticValue, parse_static};
    use pse_relations::{
        columnar::RelationRow, generated::authored::modeling_declarations as wire,
    };
    let bundle = pse_runtime::authoring_driver::document::load_package(
        &repository().join("packages/reference/data/gross-sadowski-2001"),
        &owner.registry,
        Default::default(),
        owner
            .sessions
            .validation_context(&owner.registry)
            .unwrap()
            .as_ref(),
    )
    .unwrap();
    let row = wire::Row::rows(&bundle.batches[&wire::RELATION_ID])
        .unwrap()
        .into_iter()
        .find(|row| row.name == "normal_alkanes")
        .unwrap();
    let StaticValue::Set(values) =
        parse_static(row.value.binding.unwrap().expression.as_deref().unwrap()).unwrap()
    else {
        panic!("bank membership is not explicit");
    };
    values
        .into_iter()
        .map(|value| {
            let StaticValue::Expression(expression) = value else {
                panic!("a bank member is not a reference");
            };
            pse_authoring::dsl::render_expr(&expression)
        })
        .collect()
}
fn members(bank: &[String], n: usize) -> Vec<String> {
    assert!(n <= bank.len(), "the workload exceeds the published bank");
    bank[..n].to_vec()
}

/// The bench package's documents: its manifest, whose dependency on `pse.physical` lets its
/// model name the physical quantity types, and its authored model and tests for the
/// registered workloads.
fn bench_package(seed: &[String], workloads: &[Value]) -> BTreeMap<String, String> {
    let manifest = format!(
        "[package]\n\
         package_id = \"{PACKAGE}\"\n\
         name = \"pse.bench-modeling-preparation\"\n\
         version = \"1.0.0\"\n\
         kind = \"model\"\n\
         id_policy = \"named\"\n\
         dependencies = [\n\
         {{ package_id = \"01a0e1461e93764398a669e208f425e7\", version_req = {{ operator = \"exact\", major = 1, minor = 0, patch = 0 }} }},\n\
         {{ package_id = \"01a0e169482c760bbd985b849a06417d\", version_req = {{ operator = \"exact\", major = 1, minor = 0, patch = 0 }} }},\n\
         {{ package_id = \"ce317b088cc74ac89873845b0c086ea8\", version_req = {{ operator = \"exact\", major = 1, minor = 0, patch = 0 }} }},\n\
         {{ package_id = \"b27409be5572b8712e47db271fae28cd\", version_req = {{ operator = \"exact\", major = 1, minor = 0, patch = 0 }} }},\n\
         {{ package_id = \"01a0ef5e3bc973bb9bcd6df429506c6a\", version_req = {{ operator = \"exact\", major = 1, minor = 0, patch = 0 }} }},\n\
         ]\n\
         doc = \"M1 preparation-scaling bench package (Plan 23 H2); published normal-alkane prefixes, with no synthetic parameters.\"\n"
    );
    BTreeMap::from([
        ("package.toml".to_owned(), manifest),
        (
            "models/preparation.pse".to_owned(),
            bench_source(seed, workloads),
        ),
    ])
}

/// Authored source of the bench package for the registered workloads.
fn bench_source(seed: &[String], workloads: &[Value]) -> String {
    let mut s = "package preparation_bench {\nuse chemistry @\"1.0.0\";\nuse chem @\"1.0.0\";\nuse helmholtz @\"1.0.0\";\nuse pcsaft_data @\"1.0.0\";\nuse bt_ideal @\"1.0.0\";\nuse peng_robinson @\"1.0.0\";\n".to_owned();
    s.push_str(&format!(
        "def PcSaftState(selected:Set<chemistry.species>) {{\n\
         param components:Set<chemistry.species>=selected;\n\
         child phase:helmholtz.HelmholtzPhase=helmholtz.HomogeneousPhase(selected=selected,law=pcsaft_data.potential);\n\
         param pressure:Pressure={PRESSURE};\n\
         var h_residual:DeltaH;\n\
         var ln_phi[j in components]:Scalar;\n\
         eq pressure_binding:phase.pressure==pressure;\n\
         eq enthalpy_binding:h_residual==phase.h_residual;\n\
         eq fugacity_binding[j in components]:ln_phi[j]==phase.ln_phi[j];\n\
         annotation start phase.rho(34.5{{mol/m^3}});\n\
         annotation nominal phase.rho(40{{mol/m^3}});\n\
         }}\n"
    ));
    for workload in workloads {
        let name = test_name(workload);
        match workload["model"].as_str().unwrap() {
            "pcsaft" => {
                let n = workload["components"].as_u64().unwrap() as usize;
                let selected = members(seed, n);
                s.push_str(&format!(
                    "set members_{n:02}:Set<chemistry.species>={{{}}};\n",
                    selected.join(",")
                ));
                let amounts: String = selected
                    .iter()
                    .map(|j| format!("fix root.phase.amount[{j}]={}{{mol}};", 1.0 / n as f64))
                    .collect();
                s.push_str(&format!(
                    "test {name} fixture {{dof 0; route steady; procedure solve; fix root.phase.T={TEMPERATURE}; {amounts}}} {{\n\
                     child root:PcSaftState=PcSaftState(selected=members_{n:02});\n}}\n"
                ));
            }
            "peng-robinson" => {
                // The seed's `density_inline`/`density_nested` state: IDAES 2.13 BT_PR vapor.
                let realization = match workload["realization"].as_str().unwrap() {
                    "inline" => "InlineDensity",
                    "nested" => "NestedDensity",
                    other => panic!("unregistered realization {other}"),
                };
                s.push_str(&format!(
                    "test {name} fixture {{dof 0; route steady; procedure solve; fix root.T=450{{K}}; fix root.target_pressure={PRESSURE}; fix root.amount[chem.benzene]=0.5{{mol}}; fix root.amount[chem.toluene]=0.5{{mol}}; value root.density_lower=1; value root.density_upper=100; value root.density_start=27;}} {{\n\
                     child root:peng_robinson.DensityRoot=peng_robinson.{realization}(selected=bt_ideal.aromatics,pkg=eos_data.bt_pr_parameters,law=eos_data.potential);\n}}\n"
                ));
            }
            other => panic!("unregistered model {other}"),
        }
    }
    s.push_str("}\n");
    s
}

fn test_name(workload: &Value) -> String {
    workload["id"].as_str().unwrap().replace('-', "_")
}

/// The workload's authored test in the bench package.
async fn case(package: &ModelingPackage, workload: &Value) -> pse_modeling::DeclarationId {
    let name = test_name(workload);
    let declarations = package.declarations().await.unwrap();
    declarations
        .iter()
        .find(|r| {
            r.name == name
                && r.value.kind.as_str() == "test"
                && declarations
                    .iter()
                    .any(|p| Some(p.declaration_id) == r.parent_id && p.name == "preparation_bench")
        })
        .unwrap()
        .declaration_id
}

/// Integer observation of a finished or refused preparation.
fn count<T: TryInto<i64>>(value: T) -> Value {
    value.try_into().map_or(Value::Null, Value::from)
}

/// Prepare one case cold: specialization and lowering, case resolution, then the derivative
/// programs. Returns the timed duration and the case record; a typed refusal is a result.
async fn prepare(
    owner: &WorkflowRuntime,
    package: &ModelingPackage,
    case: pse_modeling::DeclarationId,
    workload: &Value,
    stage: &str,
    phases: &phases::Phases,
) -> (Duration, Value) {
    let limits = pse_modeling::Limits {
        items: 1_000_000,
        body_slots: workload["body_slots"].as_u64().map(|v| v as usize),
        body_occurrences: workload["body_occurrences"].as_u64().map(|v| v as usize),
        ..Default::default()
    };
    let cancel = CancelSource::new();
    owner.runtime.reset_observation_peak();
    let before = owner.runtime.math().preparations();
    phases.reset();
    let mut seconds = serde_json::Map::new();
    let started = Instant::now();
    let outcome: Result<Value, (&str, WorkflowError)> = async {
        let mut analysis = package
            .declared_execution(
                case,
                Default::default(),
                Default::default(),
                Default::default(),
                limits,
                &cancel,
            )
            .await
            .map_err(|e| ("specialization_lowering", e))?
            .analysis;
        if stage == "value" {
            let path = if workload["model"] == "pcsaft" {
                "root.pressure"
            } else {
                "root.target_pressure"
            };
            analysis.case.values.insert(path.into(), 100100.);
        }
        if stage == "structural" {
            let path = if workload["model"] == "pcsaft" {
                "root.phase.rho"
            } else {
                "root.rho"
            };
            analysis
                .case
                .variables
                .entry(path.into())
                .or_default()
                .upper = Some(Some(50000.));
        }
        seconds.insert(
            "specialization_lowering".into(),
            started.elapsed().as_secs_f64().into(),
        );
        let phase = Instant::now();
        let prepared = package
            .prepare_diagnostics(&analysis, &cancel)
            .await
            .map_err(|e| ("case_resolution", e))?;
        seconds.insert(
            "case_resolution".into(),
            phase.elapsed().as_secs_f64().into(),
        );
        let phase = Instant::now();
        owner
            .runtime
            .math()
            .assemble(prepared.model.case.clone())
            .await
            .map_err(|e| ("derivative_programs", WorkflowError::from(e)))?;
        seconds.insert(
            "derivative_programs".into(),
            phase.elapsed().as_secs_f64().into(),
        );
        let admitted = &prepared.model.model.compiled().admitted;
        let bodies: Vec<_> = admitted
            .bodies
            .values()
            .map(|b| b.math())
            .chain(
                admitted
                    .implicit_systems()
                    .flat_map(|i| i.bodies().map(|b| b.math())),
            )
            .collect();
        let plan = &prepared.model.case.compiled().plan;
        Ok(json!({
            "bodies": bodies.len(),
            "formal_slots": {
                "max": bodies.iter().map(|b| b.slot_count()).max(),
                "total": bodies.iter().map(|b| b.slot_count()).sum::<usize>(),
            },
            "body_occurrences": {
                "max": bodies.iter().map(|b| b.occurrence_count()).max(),
                "total": bodies.iter().map(|b| b.occurrence_count()).sum::<usize>(),
            },
            "columns": plan.columns().len(),
            "rows": plan.structure().rows().len(),
        }))
    }
    .await;
    let elapsed = started.elapsed();
    let after = owner.runtime.math().preparations();
    seconds.insert("total".into(), elapsed.as_secs_f64().into());
    let mut record = json!({
        "experiment": "modeling_preparation",
        "id": workload["id"],
        "stage": stage,
        "compiler_phases": phases.report(1),
        "preparations": {
            "views": after.views - before.views,
            "observations": after.observations - before.observations,
            "rebuilt": after.rebuilt - before.rebuilt,
            "shared": after.shared - before.shared,
        },
        "workload": workload,
        "seconds": seconds,
        "derivative_order": "first",
        "pool_peak_bytes": count(owner.runtime.observation_peak_bytes()),
        "process_peak_rss_bytes": owner.runtime.report().unwrap().process_peak_rss_bytes,
        "scope": "specialization, lowering, case resolution and first-order derivative program assembly; ordered cold, warm, value and structural changes; no solve",
        "memory_scope": "pool observation of this preparation; process-lifetime VmHWM, per case only when PSE_PREPARATION_CASE selects one case per process",
    });
    match outcome {
        Ok(prepared) => {
            record["outcome"] = "prepared".into();
            record["prepared"] = prepared;
        }
        Err((phase, error)) => {
            record["outcome"] = "refused".into();
            record["refusal"] = json!({
                "phase": phase,
                "message": error.to_string(),
                "diagnostic": serde_json::to_value(error.boundary_diagnostic()).unwrap(),
            });
        }
    }
    (elapsed, record)
}

fn preparation(c: &mut Criterion) {
    use tracing_subscriber::prelude::*;
    let phases = phases::Phases::default();
    tracing_subscriber::registry()
        .with(phases.clone())
        .try_init()
        .unwrap();
    let registered: Value = serde_json::from_str(
        &std::fs::read_to_string(repository().join(".config/preparation-cases.json")).unwrap(),
    )
    .unwrap();
    let workloads = registered["workloads"].as_array().unwrap().clone();
    let selected = std::env::var("PSE_PREPARATION_CASE").ok();
    if let Some(id) = &selected {
        assert!(
            workloads.iter().any(|w| w["id"] == id.as_str()),
            "unregistered preparation workload {id}"
        );
    }
    // Criterion measures under `--bench`; otherwise it tests each function once, and only
    // the registered smoke workloads run unless one case is selected.
    let measuring = std::env::args().any(|a| a == "--bench");
    let output = std::env::var_os("PSE_PREPARATION_OUTPUT").map(PathBuf::from);
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    if let Some(workload) = selected
        .as_deref()
        .and_then(|id| workloads.iter().find(|w| w["id"] == id))
        .filter(|w| w["model"] == "scalar-k4")
    {
        k4_preparation::measure(c, workload, output.as_deref(), &phases);
        return;
    }
    if selected.is_none() {
        for workload in workloads
            .iter()
            .filter(|w| w["model"] == "scalar-k4" && (measuring || w["smoke"] == true))
        {
            k4_preparation::measure(c, workload, output.as_deref(), &phases);
        }
    }
    // Workloads take deterministic prefixes of one declared published bank.
    let bench = {
        let owner = WorkflowRuntime::with_threads(std::num::NonZeroUsize::new(1).unwrap()).unwrap();
        bench_package(
            &bank_members(&owner),
            &workloads
                .iter()
                .filter(|w| w["model"] != "scalar-k4")
                .cloned()
                .collect::<Vec<_>>(),
        )
    };
    let mut group = c.benchmark_group("modeling_preparation");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(1))
        .measurement_time(Duration::from_secs(1));
    for workload in workloads.iter().filter(|w| match selected.as_deref() {
        Some(id) => w["id"] == id,
        None => (measuring || w["smoke"] == true) && w["model"] != "scalar-k4",
    }) {
        let id = workload["id"].as_str().unwrap();
        let mut records = Vec::new();
        group.bench_function(id, |b| {
            b.iter_custom(|iterations| {
                let mut timed = Duration::ZERO;
                for _ in 0..iterations {
                    let owner =
                        WorkflowRuntime::with_threads(std::num::NonZeroUsize::new(1).unwrap())
                            .unwrap();
                    let package = executor.block_on(seed(&owner, &bench));
                    let case = executor.block_on(case(&package, workload));
                    let mut stages = Vec::new();
                    for stage in ["cold", "warm", "value", "structural"] {
                        let (elapsed, record) = executor
                            .block_on(prepare(&owner, &package, case, workload, stage, &phases));
                        timed += elapsed;
                        stages.push(record);
                    }
                    records.push(
                        json!({"experiment":"modeling_preparation", "id":id, "stages":stages}),
                    );
                }
                timed
            });
        });
        // Structure and outcome are deterministic; phase clocks average every iteration,
        // Criterion's warm-up included. Criterion's own report holds the timing statistics.
        let mut record = records.last().unwrap().clone();
        for (index, stage) in record["stages"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .enumerate()
        {
            for (phase, value) in stage["seconds"].as_object_mut().unwrap() {
                let clocks: Vec<_> = records
                    .iter()
                    .filter_map(|r| r["stages"][index]["seconds"][phase.as_str()].as_f64())
                    .collect();
                *value = (clocks.iter().sum::<f64>() / clocks.len() as f64).into();
            }
        }
        record["iterations"] = records.len().into();
        println!("{record}");
        if let Some(directory) = &output {
            std::fs::create_dir_all(directory).unwrap();
            std::fs::write(
                directory.join(format!("{id}.json")),
                serde_json::to_vec_pretty(&record).unwrap(),
            )
            .unwrap();
        }
    }
    group.finish();
}

fn configuration() -> Criterion {
    match std::env::var_os("PSE_PREPARATION_OUTPUT") {
        Some(output) => {
            Criterion::default().output_directory(&PathBuf::from(output).join("criterion"))
        }
        None => Criterion::default(),
    }
}
criterion_group! {name=benches;config=configuration();targets=preparation}
criterion_main!(benches);
