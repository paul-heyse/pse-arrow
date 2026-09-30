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
//! untimed, so every timed preparation is cold; only the process-global Symbolica formal
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

/// The bench's own authored test package. It is never a reference package: its synthetic
/// species and parameter rows exist only to scale the seed PC-SAFT state beyond the seed's
/// own species until the DM1 data bank supplies real ones.
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
    let documents = |names: &[&str], bench: Option<&BTreeMap<String, String>>| {
        let mut bundles = names
            .iter()
            .map(|name| load_package(&root.join(name), &owner.registry, Default::default()))
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
                )
                .unwrap(),
            );
        }
        load_bundles_owned(&bundles, &owner.registry, &pool, &owner.cancel).unwrap()
    };
    let runtime = Runtime::from_shared(
        owner.runtime.clone(),
        owner.registry.clone(),
        owner.sessions.clone(),
    );
    let physical = runtime
        .physical_from_documents(&documents(&["physical"], None), &owner.cancel)
        .await
        .unwrap();
    runtime
        .modeling_from_documents(
            &documents(
                &[
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
        .unwrap()
}

/// The seed PC-SAFT segment rows, keyed by species, in declaration order, read from the
/// seed-data package's source IR without admitting the closure.
fn seed_segments(registry: &pse_schema::Registry) -> Vec<(String, Vec<String>)> {
    use pse_relations::{columnar::RelationRow, generated::authored::modeling_declarations as wire};
    let bundle = pse_runtime::authoring_driver::document::load_package(
        &repository().join("packages/reference/seed-data"),
        registry,
        Default::default(),
    )
    .unwrap();
    wire::Row::rows(&bundle.batches[&wire::RELATION_ID])
        .unwrap()
        .into_iter()
        .filter_map(|row| row.value.dataset)
        .filter(|dataset| dataset.target == "pcsaft.segments")
        .flat_map(|dataset| {
            dataset.rows.into_iter().map(|row| {
                let cell = |cell| pse_authoring::language::render_cell(cell).unwrap();
                (cell(&row.keys[0]), row.values.iter().map(cell).collect())
            })
        })
        .collect()
}

/// Scale a numeric cell, keeping its unit literal.
fn scaled(cell: &str, factor: f64) -> String {
    let (number, unit) = cell.split_at(cell.find('{').unwrap_or(cell.len()));
    format!("{}{unit}", number.trim().parse::<f64>().unwrap() * factor)
}

/// The `n` members of an N-component state: the seed species first, then synthetic ones.
fn members(seed: &[(String, Vec<String>)], n: usize) -> Vec<String> {
    seed.iter()
        .map(|(key, _)| key.clone())
        .chain((seed.len() + 1..).map(|k| format!("synthetic_{k:02}")))
        .take(n)
        .collect()
}

/// The bench package's documents: its manifest, whose dependency on `pse.physical` lets its
/// model name the physical quantity types, and its authored model and tests for the
/// registered workloads.
fn bench_package(seed: &[(String, Vec<String>)], workloads: &[Value]) -> BTreeMap<String, String> {
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
         doc = \"M1 preparation-scaling bench package (Plan 23 H2); synthetic rows are not physical data.\"\n"
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
fn bench_source(seed: &[(String, Vec<String>)], workloads: &[Value]) -> String {
    let largest = workloads
        .iter()
        .filter_map(|w| w["components"].as_u64())
        .max()
        .unwrap_or(0) as usize;
    let all = members(seed, largest.max(seed.len()));
    let mut s = format!(
        "package preparation_bench {{\n\
         use chemistry @\"1.0.0\";\nuse chem @\"1.0.0\";\nuse helmholtz @\"1.0.0\";\n\
         use pcsaft @\"1.0.0\";\nuse peng_robinson @\"1.0.0\";\nuse provenance @\"1.0.0\";\n\
         entity provenance.source bench_rows {{ title = \"SYNTHETIC M1 preparation-bench rows: deterministic perturbations of the seed PC-SAFT segment rows and zero binary interactions, as in the seed; not physical data\" }}\n"
    );
    let synthetic = &all[seed.len()..];
    for name in synthetic {
        s.push_str(&format!("entity chemistry.species {name} {{}}\n"));
    }
    if !synthetic.is_empty() {
        // Species k repeats seed row (k-1) mod |seed|, every cell scaled by 1+0.005·(k-|seed|).
        s.push_str("dataset synthetic_segments:pcsaft.segments provenance(bench_rows, provenance.Role.synthetic) {\n");
        for (i, name) in synthetic.iter().enumerate() {
            let (_, base) = &seed[i % seed.len()];
            let factor = 1.0 + 0.005 * (i + 1) as f64;
            let cells: Vec<_> = base.iter().map(|c| scaled(c, factor)).collect();
            s.push_str(&format!("[{name}]=[{}];\n", cells.join(",")));
        }
        s.push_str("}\n");
        s.push_str("dataset synthetic_interactions:pcsaft.interaction provenance(bench_rows, provenance.Role.synthetic) {\n");
        for i in &all {
            for j in &all {
                if synthetic.contains(i) || synthetic.contains(j) {
                    s.push_str(&format!("[{i},{j}]=[0];\n"));
                }
            }
        }
        s.push_str("}\n");
    }
    s.push_str(&format!(
        "def PcSaftState(selected:Set<chemistry.species>) {{\n\
         param components:Set<chemistry.species>=selected;\n\
         child phase:helmholtz.HelmholtzPhase=helmholtz.HomogeneousPhase(selected=selected,law=pcsaft.potential);\n\
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
                    "test {name} fixture {{dof 0; run steady; fix root.phase.T={TEMPERATURE}; {amounts}}} {{\n\
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
                    "test {name} fixture {{dof 0; run steady; fix root.T=450{{K}}; fix root.target_pressure={PRESSURE}; fix root.amount[chem.benzene]=0.5{{mol}}; fix root.amount[chem.toluene]=0.5{{mol}}; value root.density_lower=1; value root.density_upper=100; value root.density_start=27;}} {{\n\
                     child root:peng_robinson.DensityRoot=peng_robinson.{realization}(selected=chem.aromatics);\n}}\n"
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
fn case(package: &ModelingPackage, workload: &Value) -> pse_modeling::DeclarationId {
    let name = test_name(workload);
    package
        .declarations()
        .iter()
        .find(|r| {
            r.name == name
                && r.value.kind.as_str() == "test"
                && package
                    .declarations()
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
) -> (Duration, Value) {
    let limits = pse_modeling::Limits {
        items: 1_000_000,
        body_slots: workload["body_slots"].as_u64().map(|v| v as usize),
        body_occurrences: workload["body_occurrences"].as_u64().map(|v| v as usize),
        ..Default::default()
    };
    let cancel = CancelSource::new();
    owner.runtime.reset_observation_peak();
    let mut seconds = serde_json::Map::new();
    let started = Instant::now();
    let outcome: Result<Value, (&str, WorkflowError)> = async {
        let analysis = package
            .declared_analysis(
                case,
                pse_relations::generated::enums::ModelingAnalysisRoute::Steady,
                Default::default(),
                Default::default(),
                Default::default(),
                limits,
                &cancel,
            )
            .await
            .map_err(|e| ("specialization_lowering", e))?;
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
            .map(|b| &b.math)
            .chain(
                admitted
                    .implicit
                    .values()
                    .flat_map(|i| i.bodies().map(|b| &b.math)),
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
    seconds.insert("total".into(), elapsed.as_secs_f64().into());
    let mut record = json!({
        "experiment": "modeling_preparation",
        "id": workload["id"],
        "workload": workload,
        "seconds": seconds,
        "derivative_order": "first",
        "pool_peak_bytes": count(owner.runtime.observation_peak_bytes()),
        "process_peak_rss_bytes": owner.runtime.report().unwrap().process_peak_rss_bytes,
        "scope": "cold specialization, lowering, case resolution (starts, nested registrations, bound structure) and first-order derivative program assembly; no solve",
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
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    // The synthetic rows derive from the seed's own PC-SAFT rows, read once.
    let bench = bench_package(
        &seed_segments(&pse_schema::shared_registry().unwrap()),
        &workloads,
    );
    let mut group = c.benchmark_group("modeling_preparation");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(1))
        .measurement_time(Duration::from_secs(1));
    for workload in workloads.iter().filter(|w| match selected.as_deref() {
        Some(id) => w["id"] == id,
        None => measuring || w["smoke"] == true,
    }) {
        let id = workload["id"].as_str().unwrap();
        let mut records = Vec::new();
        group.bench_function(id, |b| {
            b.iter_custom(|iterations| {
                let mut timed = Duration::ZERO;
                for _ in 0..iterations {
                    let owner = WorkflowRuntime::new().unwrap();
                    let package = executor.block_on(seed(&owner, &bench));
                    let case = case(&package, workload);
                    let (elapsed, record) =
                        executor.block_on(prepare(&owner, &package, case, workload));
                    timed += elapsed;
                    records.push(record);
                }
                timed
            });
        });
        // Structure and outcome are deterministic; phase clocks average every iteration,
        // Criterion's warm-up included. Criterion's own report holds the timing statistics.
        let mut record = records.last().unwrap().clone();
        for (phase, value) in record["seconds"].as_object_mut().unwrap() {
            let clocks: Vec<_> = records
                .iter()
                .filter_map(|r| r["seconds"][phase.as_str()].as_f64())
                .collect();
            *value = (clocks.iter().sum::<f64>() / clocks.len() as f64).into();
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

criterion_group!(benches, preparation);
criterion_main!(benches);
