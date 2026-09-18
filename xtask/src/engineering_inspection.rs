// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Current engineering outcomes through native compilation and cold Delta inspection.
#[path = "../../tests/support/engineering_expectations.rs"]
mod expectations;
#[path = "../../tests/support/engineering_sources.rs"]
mod sources;
use crate::inspection_fixture::environment::Environment;
use anyhow::{Context, Result, ensure};
use pse_catalog::delta::publication::PublicationRoot;
use pse_ids::SemanticId;
use pse_relations::generated::{enums::PublicationKind, normalized};
use serde::Serialize;
use std::{collections::BTreeMap, path::Path, process::Command, time::Instant};
#[derive(Serialize)]
struct Measurement {
    label: String,
    publication: PublicationRoot,
    state_template: SemanticId,
    expected_states: usize,
    planning_seconds: f64,
    execution_seconds: f64,
    rust_reopen_seconds: f64,
    memory_limit_bytes: usize,
    compile_peak_bytes: usize,
    reopen_peak_bytes: usize,
    process_peak_rss_bytes: Option<u64>,
    threads: usize,
    partitions: usize,
    batch_size: usize,
    relation_rows: BTreeMap<String, usize>,
}

pub(crate) fn run(root: &Path, output: &Path, selected: Option<&str>) -> Result<()> {
    std::fs::create_dir(output)
        .with_context(|| format!("creating new engineering store {}", output.display()))?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    for (label, unit, state) in [
        ("heater-ftpx", sources::HEATER, sources::FTPX),
        ("heater-fctp", sources::HEATER, sources::FCTP),
        ("mixer-ftpx", sources::MIXER, sources::FTPX),
        ("mixer-fctp", sources::MIXER, sources::FCTP),
    ] {
        if selected.is_some_and(|selected| selected != label) {
            continue;
        }
        let path = output.join(label);
        std::fs::create_dir(&path)?;
        println!("engineering inspection: {label} source-to-native-math and cold Delta admission");
        let measurement = runtime.block_on(produce(root, &path, label, unit, state))?;
        let mut json = serde_json::to_vec_pretty(&measurement)?;
        json.push(b'\n');
        std::fs::write(path.join("engineering.json"), json)?;
        python(root, &path)?;
        println!("engineering inspection: {label} passed Rust and Python");
    }
    Ok(())
}

fn python(root: &Path, path: &Path) -> Result<()> {
    let path = path.canonicalize()?;
    let status = Command::new("uv")
        .current_dir(root)
        .args([
            "run",
            "--no-sync",
            "pytest",
            "-m",
            "integration",
            "-n",
            "0",
            "python/pse/tests/test_engineering_inspection.py",
        ])
        .env("PSE_ENGINEERING_PUBLICATION", &path)
        .status()
        .context("running cold Python engineering inspection")?;
    ensure!(
        status.success(),
        "Python engineering inspection failed: {status}"
    );
    Ok(())
}

async fn produce(
    root: &Path,
    path: &Path,
    label: &str,
    unit: &str,
    state: &str,
) -> Result<Measurement> {
    let environment = Environment::new(path)?;
    let documents = sources::sources(root, &environment.registry, unit, state);
    let started = Instant::now();
    let plan = environment.source_plan(&documents, true).await?;
    let planning_seconds = started.elapsed().as_secs_f64();
    println!(
        "engineering inspection: {label} planned {} outputs in {planning_seconds:.3}s; publishing",
        plan.outputs().len()
    );
    let started = Instant::now();
    let publication = environment
        .publish(path, &plan, PublicationKind::Problem)
        .await?;
    let execution_seconds = started.elapsed().as_secs_f64();
    println!(
        "engineering inspection: {label} published in {execution_seconds:.3}s; opening a fresh reader"
    );
    let state_template = SemanticId::parse_hex(if state == sources::FTPX {
        "7c6cfa99af1e4477a136a4ca9d5562ec"
    } else {
        "88ad610582c04b999ec786ac9f3aebba"
    })?;
    let expected_states = if unit == sources::HEATER { 2 } else { 3 };
    let budget = environment.runtime.budget();
    let mut measurement = Measurement {
        label: label.into(),
        publication: publication.clone(),
        state_template,
        expected_states,
        planning_seconds,
        execution_seconds,
        rust_reopen_seconds: 0.,
        memory_limit_bytes: budget.memory_limit_bytes.get(),
        compile_peak_bytes: environment.runtime.report()?.pool_peak_bytes,
        reopen_peak_bytes: 0,
        process_peak_rss_bytes: None,
        threads: budget.threads.pool_threads.get(),
        partitions: budget.threads.target_partitions.get(),
        batch_size: budget.execution.batch_size,
        relation_rows: BTreeMap::new(),
    };
    drop((plan, documents, environment));
    let reader = Environment::new(path)?;
    let started = Instant::now();
    let publication = reader.open(publication).await?;
    let rows = reader.capture(&publication).await?;
    measurement.rust_reopen_seconds = started.elapsed().as_secs_f64();
    let instances = normalized::instance_bindings::View::from_checked(
        &rows[&normalized::instance_bindings::RELATION_KEY],
    )?
    .rows()?;
    ensure!(
        instances
            .iter()
            .filter(|instance| instance.template_id == state_template)
            .count()
            == expected_states,
        "state count differs from engineering example"
    );
    expectations::check(
        &rows,
        &instances,
        state_template,
        state == sources::FCTP,
        unit == sources::HEATER,
    );
    measurement.relation_rows = rows
        .iter()
        .map(|(key, value)| (key.qualified_name(), value.batch().num_rows()))
        .collect();
    let resources = reader.runtime.report()?;
    measurement.reopen_peak_bytes = resources.pool_peak_bytes;
    measurement.process_peak_rss_bytes = resources.process_peak_rss_bytes;
    Ok(measurement)
}
