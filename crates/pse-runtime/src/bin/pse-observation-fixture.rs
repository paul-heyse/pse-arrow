// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-built cooperating observation fixture, prepared only for its declared consumer.
use pse_backend_native::solve::{Event, EventSource, Metric, Progress};
use pse_diagnostics::observation::{Outcome, Policy, Relationship};
use pse_runtime::{
    observation::{Diagnostics, read_checkpoint},
    tooling::{frozen_inspection::FrozenEvidenceSet, review_pack::PackReader, telemetry},
};
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeMap,
    io::Read as _,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
fn wait(path: &Path) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(60);
    while !path.exists() {
        if Instant::now() >= deadline {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("fixture rendezvous {}", path.display()),
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}
fn verify_portable(root: &Path) -> Result<()> {
    // The existing pack reader alone resolves this explicit relocated directory. Neither
    // original checkpoint/capture paths nor the still-live observation owner are inputs.
    let reader = PackReader::open(&root.join("portable-pack"), 16 << 20)?;
    let frozen = reader.read_frozen()?;
    let analysis = reader.read_analysis()?;
    let mut raw = Vec::new();
    reader.open_member("timeline")?.read_to_end(&mut raw)?;
    let mut original_paths_available = root.join("frozen.json").try_exists()?;
    for output in frozen.captures.iter().flat_map(|capture| &capture.outputs) {
        original_paths_available |= output.path.try_exists()?;
    }
    for entry in std::fs::read_dir(root.join("checkpoints"))? {
        let entry = entry?;
        original_paths_available |= entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "json");
    }
    if original_paths_available || raw.is_empty() {
        return Err(std::io::Error::other("portable fixture requires absent original evidence paths and a nonempty selected raw member").into());
    }
    if !frozen
        .checkpoints
        .iter()
        .flat_map(|image| &image.operations)
        .any(|operation| operation.id == analysis.request.operation)
    {
        return Err(std::io::Error::other(
            "portable analysis lost the original supplied operation identity",
        )
        .into());
    }
    let digest: String = Sha256::digest(&raw)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let response = serde_json::json!({"operation":analysis.request.operation,"captures":frozen.captures.len(),"raw_bytes":raw.len(),"raw_sha256":digest,"original_paths_available":original_paths_available});
    std::fs::write(root.join("portable.ready"), serde_json::to_vec(&response)?)?;
    Ok(())
}
fn push(progress: &Progress, ordinal: u64) {
    progress.push(Event {
        source: Some(EventSource::new(
            "nonlinear_iteration",
            Some("physical_model"),
            Some("scaled_native"),
        )),
        phase: "native iteration".into(),
        elapsed: Duration::from_millis(ordinal),
        values: BTreeMap::from([
            ("iteration".into(), Metric::Integer(ordinal as i64)),
            ("obj_value".into(), Metric::Real(17.25)),
            ("inf_pr".into(), Metric::Real(f64::INFINITY)),
            (
                "native_missing".into(),
                Metric::Unavailable(
                    pse_model::generated::enums::EvidenceUnavailableReason::NotComputed,
                ),
            ),
        ]),
        incumbent: None,
    });
}
fn wait_for_finish(root: &Path, owner: &Diagnostics, progress: &Progress) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(60);
    while !root.join("finish").exists() {
        if root.join("advance").exists() && !root.join("advanced.ready").exists() {
            push(progress, 4);
            owner.publish()?;
            std::fs::write(root.join("advanced.ready"), [])?;
        }
        if root.join("verify-portable").exists() && !root.join("portable.ready").exists() {
            verify_portable(root)?;
        }
        if Instant::now() >= deadline {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "fixture finish rendezvous",
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}
fn main() -> Result<()> {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "observation fixture needs its explicitly owned output directory",
            )
        })?;
    // Fixed executable composition consumes only the launcher's existing resolved settings.
    // An unavailable optional exporter must not become checkpoint or scientific ownership.
    let telemetry = telemetry::Host::from_environment("pse-observation-fixture")?;
    if telemetry.is_some() {
        tracing::info_span!("pse.fixture.optional_export", pse.owner.stage = true).in_scope(|| {});
    }
    let owner = Diagnostics::new(
        Policy {
            record_capacity: 1,
            summary_retention: Duration::from_millis(200),
            ..Policy::default()
        },
        root.join("checkpoints"),
    );
    let operation = owner.hub().start("cooperating native progress owner");
    let context = operation.context();
    context.declare_summary("original numerical checkpoints, not permission");
    let _entered = context.enter();
    let preparation = context.child("original preparation owner", Relationship::Nested);
    let preparing = preparation
        .context()
        .phase("native preparation publication");
    preparation.context().count("prepared.items", "items", 1);
    let waiter = context.child("independent waiting consumer", Relationship::DependencyWait);
    waiter
        .context()
        .link(&preparation.context(), Relationship::SharedProduction);
    let waiting = waiter
        .context()
        .phase("waiting for original shared preparation");
    let progress = Progress::new(1);
    for ordinal in 1..=3 {
        push(&progress, ordinal);
    }
    owner.publish()?;
    let frozen = FrozenEvidenceSet {
        checkpoints: vec![read_checkpoint(&owner.checkpoint_path())?],
        ..Default::default()
    };
    std::fs::write(root.join("frozen.json"), serde_json::to_vec(&frozen)?)?;
    let evidence = serde_json::json!({"checkpoint":owner.checkpoint_path(),"operation":context.id(),"preparation":preparation.context().id(),"waiter":waiter.context().id(),"event":pse_runtime::workflow::ProgressEventDocument::from(progress.current().ok_or_else(|| std::io::Error::other("fixture current source missing"))?),"dropped":progress.snapshot().1});
    std::fs::write(root.join("source.json"), serde_json::to_vec(&evidence)?)?;
    std::fs::write(root.join("active.ready"), [])?;
    wait_for_finish(&root, &owner, &progress)?;
    drop(waiting);
    drop(preparing);
    waiter.finish(Outcome::Completed);
    preparation.finish(Outcome::Completed);
    operation.finish(Outcome::Failed);
    owner.publish()?;
    std::fs::write(root.join("terminal.ready"), [])?;
    wait(&root.join("expire"))?;
    std::thread::sleep(Duration::from_millis(250));
    owner.publish()?;
    std::fs::write(root.join("expired.ready"), [])?;
    wait(&root.join("close"))?;
    owner.release()?;
    if let Some(host) = telemetry {
        let loss = host.loss_observer();
        let shutdown = host.shutdown();
        std::fs::write(
            root.join("telemetry.settled"),
            serde_json::to_vec(&serde_json::json!({
                "shutdown_error": shutdown.err().map(|error| error.to_string()),
                "health": loss.snapshot(),
            }))?,
        )?;
    }
    Ok(())
}
