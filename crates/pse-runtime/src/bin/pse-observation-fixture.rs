// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-built cooperating observation fixture, prepared only for its declared consumer.
use pse_backend_native::solve::{Event, EventSource, Metric, Progress};
use pse_diagnostics::observation::{Outcome, Policy};
use pse_runtime::{
    observation::{Diagnostics, read_checkpoint},
    tooling::frozen_inspection::FrozenEvidenceSet,
};
use std::{
    collections::BTreeMap,
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
    let progress = Progress::new(1);
    for ordinal in 1..=3 {
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
    owner.publish()?;
    let frozen = FrozenEvidenceSet {
        checkpoints: vec![read_checkpoint(&owner.checkpoint_path())?],
        ..Default::default()
    };
    std::fs::write(root.join("frozen.json"), serde_json::to_vec(&frozen)?)?;
    let evidence = serde_json::json!({"checkpoint":owner.checkpoint_path(),"operation":context.id(),"event":pse_runtime::workflow::ProgressEventDocument::from(progress.current().ok_or_else(|| std::io::Error::other("fixture current source missing"))?),"dropped":progress.snapshot().1});
    std::fs::write(root.join("source.json"), serde_json::to_vec(&evidence)?)?;
    std::fs::write(root.join("active.ready"), [])?;
    wait(&root.join("finish"))?;
    operation.finish(Outcome::Failed);
    owner.publish()?;
    std::fs::write(root.join("terminal.ready"), [])?;
    wait(&root.join("expire"))?;
    std::thread::sleep(Duration::from_millis(250));
    owner.publish()?;
    std::fs::write(root.join("expired.ready"), [])?;
    wait(&root.join("close"))?;
    owner.release()?;
    Ok(())
}
