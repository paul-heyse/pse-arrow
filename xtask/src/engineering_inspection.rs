// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-to-P10, cold admission and Python inspection of fresh engineering stores.

#[path = "../../tests/support/engineering_expectations.rs"]
mod expectations;
#[path = "../../tests/support/engineering_sources.rs"]
mod sources;

use crate::inspection_fixture::environment::Environment;
use anyhow::{Context, Result, ensure};
use pse_catalog::{Snapshot, snapshot::ManifestRef};
use pse_compiler::{
    PolicySet,
    driver::{CommitRequest, Driver, PipelineRequest},
};
use pse_ids::SemanticId;
use pse_relations::generated::{authored, normalized};
use serde::Serialize;
use std::{collections::BTreeMap, path::Path, process::Command, sync::Arc, time::Instant};

#[derive(Serialize)]
struct Measurement {
    label: String,
    p3: ManifestRef,
    p10: ManifestRef,
    state_template: SemanticId,
    expected_states: usize,
    commit_seconds: f64,
    compile_seconds: f64,
    rust_reopen_seconds: f64,
    memory_limit_bytes: usize,
    compile_peak_bytes: usize,
    reopen_peak_bytes: usize,
    process_peak_rss_bytes: Option<u64>,
    threads: usize,
    partitions: usize,
    batch_size: usize,
    stage_rows: BTreeMap<String, BTreeMap<String, usize>>,
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
        println!("engineering inspection: {label} source-to-P10 and cold Rust admission");
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
        .env("PSE_ENGINEERING_STORE", &path)
        .status()
        .context("running cold Python engineering inspection")?;
    ensure!(
        status.success(),
        "Python engineering inspection failed: {status}"
    );
    Ok(())
}

async fn commit(
    environment: &Environment,
    driver: &mut Driver,
    root: &Path,
    unit: &str,
    state: &str,
) -> Result<Arc<Snapshot>> {
    let documents = sources::sources(root, environment.catalog.registry(), unit, state);
    let result = driver
        .commit(
            CommitRequest {
                reference: pse_catalog::RefName::parse("engineering")?,
                revision_ids: None,
                base: None,
                documents,
                changes: None,
                header: authored::change_sets::Row {
                    change_set_id: pse_authoring::ids::uuid_v7(),
                    base_revision_id: SemanticId::NIL,
                    author: "engineering inspection".into(),
                    message: "fresh current source workflow".into(),
                    created_at: 1_000_000_000,
                },
            },
            &environment.cancel,
        )
        .await?;
    ensure!(
        result.validation.error_count() == 0,
        "source admission reported errors"
    );
    result
        .tip
        .context("source admission omitted its actual tip")
}

async fn produce(
    root: &Path,
    path: &Path,
    label: &str,
    unit: &str,
    state: &str,
) -> Result<Measurement> {
    let environment = Environment::new(path)?;
    let mut driver = Driver::new(Arc::clone(&environment.catalog))?;
    let started = Instant::now();
    let source = commit(&environment, &mut driver, root, unit, state).await?;
    let commit_seconds = started.elapsed().as_secs_f64();
    println!("{label}: source admitted in {commit_seconds:.3}s; compiling P3-P10");
    let started = Instant::now();
    let report = driver
        .run(
            PipelineRequest {
                through: "P10".into(),
                snapshot: source,
                policies: PolicySet::default(),
                reuse: false,
            },
            &environment.cancel,
        )
        .await?;
    let compile_seconds = started.elapsed().as_secs_f64();
    println!("{label}: P10 compiled in {compile_seconds:.3}s; checking and cold reopening");
    let p3 = &report.stages.first().context("P3 result absent")?.snapshot;
    let p10 = &report.stages.last().context("P10 result absent")?.snapshot;
    let state_template = SemanticId::parse_hex(if state == sources::FTPX {
        "7c6cfa99af1e4477a136a4ca9d5562ec"
    } else {
        "88ad610582c04b999ec786ac9f3aebba"
    })?;
    let expected_states = if unit == sources::HEATER { 2 } else { 3 };
    check(
        p3,
        p10,
        state_template,
        state == sources::FCTP,
        unit == sources::HEATER,
        expected_states,
    )?;
    let budget = environment.runtime.budget();
    let mut measurement = Measurement {
        label: label.into(),
        p3: p3.manifest_ref(),
        p10: p10.manifest_ref(),
        state_template,
        expected_states,
        commit_seconds,
        compile_seconds,
        rust_reopen_seconds: 0.,
        memory_limit_bytes: budget.memory_limit_bytes.get(),
        compile_peak_bytes: environment.runtime.report()?.pool_peak_bytes,
        reopen_peak_bytes: 0,
        process_peak_rss_bytes: None,
        threads: budget.threads.pool_threads.get(),
        partitions: budget.threads.target_partitions.get(),
        batch_size: budget.execution.batch_size,
        stage_rows: report
            .stages
            .iter()
            .map(|stage| {
                (
                    stage.pass.clone(),
                    stage
                        .snapshot
                        .relations()
                        .iter()
                        .map(|(port, relation)| (port.clone(), relation.rows()))
                        .collect(),
                )
            })
            .collect(),
    };
    drop((report, driver, environment));
    reopen(path, &mut measurement, unit, state).await?;
    Ok(measurement)
}

async fn reopen(path: &Path, measurement: &mut Measurement, unit: &str, state: &str) -> Result<()> {
    let reader = Environment::new(path)?;
    let started = Instant::now();
    let reopened = reader
        .catalog
        .read_pinned_manifest(measurement.p10, &reader.cancel)
        .await?;
    measurement.rust_reopen_seconds = started.elapsed().as_secs_f64();
    let p3 =
        find_snapshot(&reopened, measurement.p3).context("cold P10 omitted its P3 dependency")?;
    check(
        &p3,
        &reopened,
        measurement.state_template,
        state == sources::FCTP,
        unit == sources::HEATER,
        measurement.expected_states,
    )?;
    let resources = reader.runtime.report()?;
    measurement.reopen_peak_bytes = resources.pool_peak_bytes;
    measurement.process_peak_rss_bytes = resources.process_peak_rss_bytes;
    println!(
        "{}: commit {:.3}s, compile {:.3}s, cold Rust {:.3}s",
        measurement.label,
        measurement.commit_seconds,
        measurement.compile_seconds,
        measurement.rust_reopen_seconds
    );
    Ok(())
}

fn check(
    p3: &Snapshot,
    p10: &Snapshot,
    state: SemanticId,
    component_flow: bool,
    heater: bool,
    count: usize,
) -> Result<()> {
    let instances = normalized::instance_bindings::View::from_checked(
        p3.relation("normalized", "instance_bindings")
            .context("actual P3 instances absent")?
            .checked(),
    )?
    .rows()?;
    ensure!(
        instances
            .iter()
            .filter(|instance| instance.template_id == state)
            .count()
            == count,
        "actual state count differs from the engineering example"
    );
    expectations::check(p10, &instances, state, component_flow, heater);
    Ok(())
}

fn find_snapshot(root: &Arc<Snapshot>, reference: ManifestRef) -> Option<Arc<Snapshot>> {
    let mut pending = vec![Arc::clone(root)];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(snapshot) = pending.pop() {
        if snapshot.manifest_ref() == reference {
            return Some(snapshot);
        }
        let actual = snapshot.manifest_ref();
        if seen.insert((actual.snapshot_id.0, actual.manifest_checksum.0)) {
            pending.extend(snapshot.parents().values().cloned());
        }
    }
    None
}

/// Terminal Plan 07 gate. The simulator integration target owns the same source
/// fixtures and assertions as ordinary tests. A missing target or incomplete backend
/// fails this command; the older P10 inspection cannot satisfy simulator acceptance.
pub(crate) fn simulator_acceptance(root: &Path, output: &Path) -> Result<()> {
    std::fs::create_dir(output).with_context(|| {
        format!(
            "creating new simulator acceptance output {}",
            output.display()
        )
    })?;
    let output = output.canonicalize()?;
    let status = Command::new("just")
        .current_dir(root)
        .args([
            "test-package",
            "pse-tests-engine",
            "--test",
            "unified_simulator",
        ])
        .env("PSE_SIMULATOR_ACCEPTANCE", &output)
        .status()
        .context("executing source-to-solve simulator acceptance")?;
    ensure!(
        status.success(),
        "simulator acceptance incomplete or failed: {status}"
    );
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
            "python/pse/tests/test_unified_simulator.py",
        ])
        .env("PSE_SIMULATOR_ACCEPTANCE", &output)
        .status()
        .context("executing cold Python simulator acceptance")?;
    ensure!(
        status.success(),
        "cold Python simulator acceptance incomplete or failed: {status}"
    );
    Ok(())
}
